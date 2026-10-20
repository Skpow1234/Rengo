//! Query execution engine

use std::collections::HashMap;

use rengo_core::{Key, RengoError, Result, Value};
use rengo_storage::{MVCCStorage, StorageEngine};

use crate::sql::aggregation::{contains_aggregate, extract_aggregates, GroupAggregationState};
use crate::sql::ast::*;

/// Query execution context
pub struct ExecutionContext {
    /// Storage engine for data access
    pub storage: Box<dyn StorageEngine>,
    /// MVCC layer for transaction support
    pub mvcc: MVCCStorage,
    /// Schema catalog
    pub catalog: SchemaCatalog,
}

/// Schema catalog for metadata management
#[derive(Debug, Clone, Default)]
pub struct SchemaCatalog {
    /// Table schemas by name
    tables: HashMap<String, TableSchema>,
}

/// Table schema definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSchema>,
    pub primary_key: Option<Vec<String>>,
    pub indexes: Vec<IndexSchema>,
}

/// Column schema definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Expression>,
    pub auto_increment: bool,
}

/// Index schema definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IndexSchema {
    pub name: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub index_type: IndexType,
    pub unique: bool,
}

/// Query executor
pub struct QueryExecutor {
    /// Execution context (storage, MVCC, catalog)
    pub context: ExecutionContext,
}

/// Query execution result
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub affected_rows: u64,
}

impl SchemaCatalog {
    /// Create a new schema catalog
    pub fn new() -> Self {
        Self { tables: HashMap::new() }
    }

    /// Add a table schema
    pub fn add_table(&mut self, schema: TableSchema) {
        self.tables.insert(schema.name.clone(), schema);
    }

    /// Get a table schema
    pub fn get_table(&self, name: &str) -> Option<&TableSchema> {
        self.tables.get(name)
    }

    /// Remove a table schema
    pub fn remove_table(&mut self, name: &str) -> Option<TableSchema> {
        self.tables.remove(name)
    }

    /// List all table names
    pub fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }
}

impl QueryExecutor {
    /// Create a new query executor
    pub fn new(context: ExecutionContext) -> Self {
        Self { context }
    }

    /// Execute a SQL statement
    pub async fn execute(&mut self, statement: Statement) -> Result<QueryResult> {
        match statement {
            Statement::CreateTable(stmt) => self.execute_create_table(stmt).await,
            Statement::DropTable(stmt) => self.execute_drop_table(stmt).await,
            Statement::CreateIndex(stmt) => self.execute_create_index(stmt).await,
            Statement::DropIndex(stmt) => self.execute_drop_index(stmt).await,
            Statement::AlterTable(stmt) => self.execute_alter_table(stmt).await,
            Statement::Insert(stmt) => self.execute_insert(stmt).await,
            Statement::Update(stmt) => self.execute_update(stmt).await,
            Statement::Delete(stmt) => self.execute_delete(stmt).await,
            Statement::Select(stmt) => self.execute_select(stmt).await,
            Statement::Begin(_) | Statement::Commit(_) | Statement::Rollback(_) => {
                // Lifecycle is owned by DatabaseNode / TransactionManager
                Ok(QueryResult {
                    columns: vec![],
                    rows: vec![],
                    affected_rows: 0,
                })
            }
        }
    }

    async fn persist_catalog_entry(&mut self, schema: &TableSchema) -> Result<()> {
        let key = Key::from_string(&format!("__catalog__table__{}", schema.name));
        let bytes = serde_json::to_vec(schema).map_err(|e| {
            RengoError::internal(format!("Failed to serialize table schema: {}", e))
        })?;
        self.context.storage.put(key, Value::new(&bytes)).await?;

        let mut names = self.context.catalog.list_tables();
        if !names.iter().any(|n| n == &schema.name) {
            names.push(schema.name.clone());
        }
        names.sort();
        let list_key = Key::from_string("__catalog__tables");
        let list_bytes = serde_json::to_vec(&names).map_err(|e| {
            RengoError::internal(format!("Failed to serialize catalog list: {}", e))
        })?;
        self.context.storage.put(list_key, Value::new(&list_bytes)).await?;
        Ok(())
    }

    async fn remove_catalog_entry(&mut self, table_name: &str) -> Result<()> {
        let key = Key::from_string(&format!("__catalog__table__{}", table_name));
        self.context.storage.delete(&key).await?;

        let names: Vec<String> = self
            .context
            .catalog
            .list_tables()
            .into_iter()
            .filter(|n| n != table_name)
            .collect();
        let list_key = Key::from_string("__catalog__tables");
        let list_bytes = serde_json::to_vec(&names).map_err(|e| {
            RengoError::internal(format!("Failed to serialize catalog list: {}", e))
        })?;
        self.context.storage.put(list_key, Value::new(&list_bytes)).await?;
        Ok(())
    }

    async fn next_row_id(&mut self, table_name: &str) -> Result<u64> {
        let seq_key = Key::from_string(&format!("__seq__{}", table_name));
        let next = match self.context.storage.get(&seq_key).await? {
            Some(value) => {
                let s = String::from_utf8_lossy(value.as_slice());
                s.parse::<u64>().unwrap_or(0) + 1
            }
            None => 1,
        };
        self.context.storage.put(seq_key, Value::from_string(&next.to_string())).await?;
        Ok(next)
    }

    /// Execute CREATE TABLE statement
    async fn execute_create_table(&mut self, stmt: CreateTableStatement) -> Result<QueryResult> {
        // Check if table already exists
        if self.context.catalog.get_table(&stmt.name).is_some() {
            if stmt.if_not_exists {
                return Ok(QueryResult {
                    columns: vec![],
                    rows: vec![],
                    affected_rows: 0,
                });
            } else {
                return Err(RengoError::Query {
                    message: format!("Table '{}' already exists", stmt.name),
                });
            }
        }

        // Convert AST column definitions to schema
        let columns = stmt
            .columns
            .into_iter()
            .map(|col| ColumnSchema {
                name: col.name,
                data_type: col.data_type,
                nullable: col.nullable,
                default_value: col.default,
                auto_increment: col.auto_increment,
            })
            .collect();

        let primary_key = stmt.constraints.iter().find_map(|c| match c {
            TableConstraint::PrimaryKey { columns } => Some(columns.clone()),
            _ => None,
        });

        // Create table schema
        let table_schema = TableSchema {
            name: stmt.name.clone(),
            columns,
            primary_key,
            indexes: vec![],
        };

        // Add to catalog and persist durable metadata
        self.context.catalog.add_table(table_schema.clone());
        self.persist_catalog_entry(&table_schema).await?;

        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows: 1,
        })
    }

    /// Execute DROP TABLE statement
    async fn execute_drop_table(&mut self, stmt: DropTableStatement) -> Result<QueryResult> {
        // Check if table exists
        if self.context.catalog.get_table(&stmt.name).is_none() {
            if stmt.if_exists {
                return Ok(QueryResult {
                    columns: vec![],
                    rows: vec![],
                    affected_rows: 0,
                });
            } else {
                return Err(RengoError::Query {
                    message: format!("Table '{}' does not exist", stmt.name),
                });
            }
        }

        // Delete row data
        let prefix = format!("{}:", stmt.name);
        let rows = self.context.storage.scan_prefix(prefix.as_bytes()).await?;
        for (key, _) in rows {
            self.context.storage.delete(&key).await?;
        }

        // Remove from catalog and durable metadata
        self.context.catalog.remove_table(&stmt.name);
        self.remove_catalog_entry(&stmt.name).await?;

        let seq_key = Key::from_string(&format!("__seq__{}", stmt.name));
        let _ = self.context.storage.delete(&seq_key).await;

        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows: 1,
        })
    }

    /// Execute INSERT statement
    async fn execute_insert(&mut self, stmt: InsertStatement) -> Result<QueryResult> {
        let schema_column_count = {
            let table_schema =
                self.context.catalog.get_table(&stmt.table_name).ok_or_else(|| {
                    RengoError::Query {
                        message: format!("Table '{}' does not exist", stmt.table_name),
                    }
                })?;
            table_schema.columns.len()
        };

        let mut affected_rows = 0;

        // Process each row
        for (row_index, values) in stmt.values.iter().enumerate() {
            let expected_columns = if let Some(ref columns) = stmt.columns {
                columns.len()
            } else {
                schema_column_count
            };

            if values.len() != expected_columns {
                return Err(RengoError::Query {
                    message: format!(
                        "Expected {} values, got {} for row {}",
                        expected_columns,
                        values.len(),
                        row_index
                    ),
                });
            }

            let mut row_data = Vec::new();
            for expr in values {
                let value = self.evaluate_expression(expr)?;
                row_data.push(value);
            }

            let row_id = self.next_row_id(&stmt.table_name).await?;
            let primary_key = Key::from_string(&format!("{}:{}", stmt.table_name, row_id));

            let serialized = self.serialize_row(&row_data)?;
            let storage_value = Value::new(&serialized);

            self.context.storage.put(primary_key, storage_value).await?;
            affected_rows += 1;
        }

        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows,
        })
    }

    /// Execute UPDATE statement
    async fn execute_update(&mut self, stmt: UpdateStatement) -> Result<QueryResult> {
        let table_schema =
            self.context.catalog.get_table(&stmt.table_name).cloned().ok_or_else(|| {
                RengoError::Query {
                    message: format!("Table '{}' does not exist", stmt.table_name),
                }
            })?;

        let mut affected_rows = 0;
        let all_keys = self.scan_table_keys(&stmt.table_name).await?;

        for key in all_keys {
            if let Some(storage_value) = self.context.storage.get(&key).await? {
                let row_data = self.deserialize_row(storage_value.as_ref())?;

                if let Some(ref where_clause) = stmt.where_clause {
                    if !self.evaluate_condition(where_clause, &table_schema, &row_data)? {
                        continue;
                    }
                }

                let mut updated_row = row_data.clone();
                for assignment in &stmt.assignments {
                    let column_index = table_schema
                        .columns
                        .iter()
                        .position(|c| c.name == assignment.column)
                        .ok_or_else(|| RengoError::Query {
                            message: format!("Column '{}' not found", assignment.column),
                        })?;

                    let new_value = self.evaluate_expression(&assignment.value)?;
                    updated_row[column_index] = new_value;
                }

                let serialized = self.serialize_row(&updated_row)?;
                let storage_value = Value::new(&serialized);
                self.context.storage.put(key, storage_value).await?;
                affected_rows += 1;
            }
        }

        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows,
        })
    }

    /// Execute DELETE statement
    async fn execute_delete(&mut self, stmt: DeleteStatement) -> Result<QueryResult> {
        let table_schema =
            self.context.catalog.get_table(&stmt.table_name).cloned().ok_or_else(|| {
                RengoError::Query {
                    message: format!("Table '{}' does not exist", stmt.table_name),
                }
            })?;

        let mut affected_rows = 0;
        let all_keys = self.scan_table_keys(&stmt.table_name).await?;

        for key in all_keys {
            if let Some(storage_value) = self.context.storage.get(&key).await? {
                let row_data = self.deserialize_row(storage_value.as_ref())?;

                if let Some(ref where_clause) = stmt.where_clause {
                    if !self.evaluate_condition(where_clause, &table_schema, &row_data)? {
                        continue;
                    }
                }

                self.context.storage.delete(&key).await?;
                affected_rows += 1;
            }
        }

        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows,
        })
    }

    /// Execute SELECT statement with full table scan
    async fn execute_select(&mut self, stmt: SelectStatement) -> Result<QueryResult> {
        // Allow `SELECT <literals>` without FROM (e.g. SELECT 1)
        if stmt.from.is_none() {
            return self.execute_select_literals(&stmt);
        }

        let table_name = stmt.from.clone().unwrap();

        let table_schema =
            self.context.catalog.get_table(&table_name).cloned().ok_or_else(|| {
                RengoError::Query {
                    message: format!("Table '{}' does not exist", table_name),
                }
            })?;

        let has_aggregates = stmt.columns.iter().any(|item| {
            if let SelectItem::Expression { expr, .. } = item {
                contains_aggregate(expr)
            } else {
                false
            }
        });

        if has_aggregates || !stmt.group_by.is_empty() {
            self.execute_select_with_aggregation(stmt, &table_schema, &table_name).await
        } else {
            self.execute_select_simple(stmt, &table_schema, &table_name).await
        }
    }

    fn execute_select_literals(&self, stmt: &SelectStatement) -> Result<QueryResult> {
        let mut columns = Vec::new();
        let mut row = Vec::new();
        for (idx, item) in stmt.columns.iter().enumerate() {
            match item {
                SelectItem::Expression { expr, alias } => {
                    let name = alias.clone().unwrap_or_else(|| format!("col{}", idx + 1));
                    columns.push(name);
                    row.push(self.evaluate_expression(expr)?);
                }
                SelectItem::Wildcard => {
                    return Err(RengoError::Query {
                        message: "SELECT * requires a FROM clause".to_string(),
                    });
                }
            }
        }
        Ok(QueryResult {
            columns,
            rows: vec![row],
            affected_rows: 0,
        })
    }

    /// Execute a simple SELECT without aggregation
    async fn execute_select_simple(
        &mut self,
        stmt: SelectStatement,
        table_schema: &TableSchema,
        table_name: &str,
    ) -> Result<QueryResult> {
        let selected_column_names = self.get_selected_column_names(&stmt.columns, table_schema)?;
        let mut result_rows = Vec::new();

        // Scan all rows in the table
        let all_keys = self.scan_table_keys(table_name).await?;

        for key in all_keys {
            if let Some(storage_value) = self.context.storage.get(&key).await? {
                let row_data = self.deserialize_row(storage_value.as_ref())?;

                // Apply WHERE filter
                if let Some(ref where_clause) = stmt.where_clause {
                    if !self.evaluate_condition(where_clause, table_schema, &row_data)? {
                        continue;
                    }
                }

                // Apply column selection
                let (_, selected_data) =
                    self.apply_column_selection(&stmt.columns, table_schema, &row_data)?;

                result_rows.push(selected_data);
            }
        }

        // Apply DISTINCT
        if stmt.distinct {
            result_rows.sort();
            result_rows.dedup();
        }

        // Apply ORDER BY
        if !stmt.order_by.is_empty() {
            result_rows.sort_by(|a, b| {
                for order_item in &stmt.order_by {
                    // Simplified: compare by column index
                    if let Expression::Column(ref col_name) = order_item.expression {
                        if let Some(col_index) =
                            selected_column_names.iter().position(|n| n == col_name)
                        {
                            if col_index < a.len() && col_index < b.len() {
                                let cmp = a[col_index].as_ref().cmp(b[col_index].as_ref());
                                if cmp != std::cmp::Ordering::Equal {
                                    return match order_item.direction {
                                        OrderDirection::Asc => cmp,
                                        OrderDirection::Desc => cmp.reverse(),
                                    };
                                }
                            }
                        }
                    }
                }
                std::cmp::Ordering::Equal
            });
        }

        // Apply LIMIT and OFFSET
        let offset = stmt.offset.unwrap_or(0) as usize;
        let limit = stmt.limit.map(|l| l as usize);

        let final_rows: Vec<_> =
            result_rows.into_iter().skip(offset).take(limit.unwrap_or(usize::MAX)).collect();

        let affected_rows = final_rows.len() as u64;

        Ok(QueryResult {
            columns: selected_column_names,
            rows: final_rows,
            affected_rows,
        })
    }

    /// Execute a SELECT with aggregation and GROUP BY
    async fn execute_select_with_aggregation(
        &mut self,
        stmt: SelectStatement,
        table_schema: &TableSchema,
        table_name: &str,
    ) -> Result<QueryResult> {
        // Extract expressions from SELECT items
        let mut select_exprs = Vec::new();
        let mut column_names = Vec::new();

        for item in &stmt.columns {
            match item {
                SelectItem::Expression { expr, alias } => {
                    select_exprs.push(expr.clone());

                    // Generate column name
                    let col_name = if let Some(alias) = alias {
                        alias.clone()
                    } else {
                        match expr {
                            Expression::Function { name, .. } => name.clone(),
                            Expression::Column(col) => col.clone(),
                            _ => "expr".to_string(),
                        }
                    };
                    column_names.push(col_name);
                }
                SelectItem::Wildcard => {
                    return Err(RengoError::Query {
                        message: "Cannot use * with aggregate functions".to_string(),
                    });
                }
            }
        }

        // Extract aggregate functions
        let aggregates = extract_aggregates(&select_exprs);

        // Determine GROUP BY columns
        let group_by_indices: Vec<usize> = stmt
            .group_by
            .iter()
            .filter_map(|expr| {
                if let Expression::Column(col_name) = expr {
                    table_schema.columns.iter().position(|c| c.name == *col_name)
                } else {
                    None
                }
            })
            .collect();

        // Extract aggregate column indices and functions
        let aggregate_funcs: Vec<(usize, crate::sql::aggregation::AggregateFunction)> = aggregates
            .iter()
            .filter_map(|(_, func, arg_expr)| {
                if let Expression::Column(col_name) = arg_expr {
                    table_schema
                        .columns
                        .iter()
                        .position(|c| c.name == *col_name)
                        .map(|idx| (idx, func.clone()))
                } else {
                    // For COUNT(*), use column 0 as a placeholder
                    Some((0, func.clone()))
                }
            })
            .collect();

        // Create aggregation state
        let mut agg_state = GroupAggregationState::new(group_by_indices.clone(), aggregate_funcs);

        // Scan all rows and accumulate
        let all_keys = self.scan_table_keys(table_name).await?;

        for key in all_keys {
            if let Some(storage_value) = self.context.storage.get(&key).await? {
                let row_data = self.deserialize_row(storage_value.as_ref())?;

                // Apply WHERE filter
                if let Some(ref where_clause) = stmt.where_clause {
                    if !self.evaluate_condition(where_clause, table_schema, &row_data)? {
                        continue;
                    }
                }

                // Accumulate for aggregation
                agg_state.accumulate_row(&row_data)?;
            }
        }

        // Finalize aggregation
        let groups = agg_state.finalize()?;

        // Build result rows
        let mut result_rows = Vec::new();
        for (group_values, aggregate_values) in groups {
            let mut row = Vec::new();

            // Add GROUP BY columns
            for val in group_values {
                row.push(val);
            }

            // Add aggregate values
            for val in aggregate_values {
                row.push(val);
            }

            // Apply HAVING filter if present
            if let Some(ref having) = stmt.having {
                // Simplified HAVING evaluation - would need more context
                // For now, skip HAVING clause evaluation
            }

            result_rows.push(row);
        }

        // Apply ORDER BY
        if !stmt.order_by.is_empty() {
            result_rows.sort_by(|a, b| {
                for order_item in &stmt.order_by {
                    if let Expression::Column(ref col_name) = order_item.expression {
                        if let Some(col_index) = column_names.iter().position(|n| n == col_name) {
                            if col_index < a.len() && col_index < b.len() {
                                let cmp = a[col_index].as_ref().cmp(b[col_index].as_ref());
                                if cmp != std::cmp::Ordering::Equal {
                                    return match order_item.direction {
                                        OrderDirection::Asc => cmp,
                                        OrderDirection::Desc => cmp.reverse(),
                                    };
                                }
                            }
                        }
                    }
                }
                std::cmp::Ordering::Equal
            });
        }

        // Apply LIMIT and OFFSET
        let offset = stmt.offset.unwrap_or(0) as usize;
        let limit = stmt.limit.map(|l| l as usize);

        let final_rows: Vec<_> =
            result_rows.into_iter().skip(offset).take(limit.unwrap_or(usize::MAX)).collect();

        let affected_rows = final_rows.len() as u64;

        Ok(QueryResult {
            columns: column_names,
            rows: final_rows,
            affected_rows,
        })
    }

    /// Execute CREATE INDEX statement
    async fn execute_create_index(&mut self, stmt: CreateIndexStatement) -> Result<QueryResult> {
        // Get table schema
        let mut table_schema = self
            .context
            .catalog
            .get_table(&stmt.table_name)
            .ok_or_else(|| RengoError::Query {
                message: format!("Table '{}' does not exist", stmt.table_name),
            })?
            .clone();

        // Check if index already exists
        if table_schema.indexes.iter().any(|idx| idx.name == stmt.name) {
            if stmt.if_not_exists {
                return Ok(QueryResult {
                    columns: vec![],
                    rows: vec![],
                    affected_rows: 0,
                });
            } else {
                return Err(RengoError::Query {
                    message: format!("Index '{}' already exists", stmt.name),
                });
            }
        }

        // Add index to schema
        table_schema.indexes.push(IndexSchema {
            name: stmt.name.clone(),
            table_name: stmt.table_name.clone(),
            columns: stmt.columns,
            index_type: stmt.index_type,
            unique: stmt.unique,
        });

        // Update catalog
        self.context.catalog.add_table(table_schema);

        // TODO: Build the index from existing data

        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows: 1,
        })
    }

    /// Execute DROP INDEX statement
    async fn execute_drop_index(&mut self, stmt: DropIndexStatement) -> Result<QueryResult> {
        // Find the table that has this index
        let tables = self.context.catalog.list_tables();
        let mut found = false;

        for table_name in tables {
            if let Some(mut table_schema) = self.context.catalog.get_table(&table_name).cloned() {
                if let Some(index_pos) =
                    table_schema.indexes.iter().position(|idx| idx.name == stmt.name)
                {
                    table_schema.indexes.remove(index_pos);
                    self.context.catalog.add_table(table_schema);
                    found = true;
                    break;
                }
            }
        }

        if !found && !stmt.if_exists {
            return Err(RengoError::Query {
                message: format!("Index '{}' does not exist", stmt.name),
            });
        }

        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows: if found { 1 } else { 0 },
        })
    }

    /// Execute ALTER TABLE statement
    async fn execute_alter_table(&mut self, stmt: AlterTableStatement) -> Result<QueryResult> {
        let mut table_schema = self
            .context
            .catalog
            .get_table(&stmt.name)
            .ok_or_else(|| RengoError::Query {
                message: format!("Table '{}' does not exist", stmt.name),
            })?
            .clone();

        match stmt.action {
            AlterTableAction::AddColumn(column_def) => {
                // Check if column already exists
                if table_schema.columns.iter().any(|c| c.name == column_def.name) {
                    return Err(RengoError::Query {
                        message: format!("Column '{}' already exists", column_def.name),
                    });
                }

                table_schema.columns.push(ColumnSchema {
                    name: column_def.name,
                    data_type: column_def.data_type,
                    nullable: column_def.nullable,
                    default_value: column_def.default,
                    auto_increment: column_def.auto_increment,
                });
            }
            AlterTableAction::AddConstraint(_constraint) => {
                // TODO: Implement constraint addition
                return Err(RengoError::Query {
                    message: "Adding constraints not yet implemented".to_string(),
                });
            }
            _ => {
                return Err(RengoError::Query {
                    message: "ALTER TABLE action not yet implemented".to_string(),
                });
            }
        }

        // Update catalog
        self.context.catalog.add_table(table_schema);

        Ok(QueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows: 1,
        })
    }

    /// Scan all keys for a table
    async fn scan_table_keys(&self, table_name: &str) -> Result<Vec<Key>> {
        let prefix = format!("{}:", table_name);
        let entries = self.context.storage.scan_prefix(prefix.as_bytes()).await?;
        Ok(entries.into_iter().map(|(key, _)| key).collect())
    }

    /// Evaluate a condition against a row
    fn evaluate_condition(
        &self,
        condition: &Expression,
        table_schema: &TableSchema,
        row_data: &[Value],
    ) -> Result<bool> {
        match condition {
            Expression::BinaryOp { left, op, right } => match op {
                // Logical ops compose conditions, not scalar values
                BinaryOperator::And => {
                    let left_bool = self.evaluate_condition(left, table_schema, row_data)?;
                    let right_bool = self.evaluate_condition(right, table_schema, row_data)?;
                    Ok(left_bool && right_bool)
                }
                BinaryOperator::Or => {
                    let left_bool = self.evaluate_condition(left, table_schema, row_data)?;
                    let right_bool = self.evaluate_condition(right, table_schema, row_data)?;
                    Ok(left_bool || right_bool)
                }
                BinaryOperator::Equal
                | BinaryOperator::NotEqual
                | BinaryOperator::LessThan
                | BinaryOperator::LessThanOrEqual
                | BinaryOperator::GreaterThan
                | BinaryOperator::GreaterThanOrEqual => {
                    let left_val = self.evaluate_expression_with_row(left, table_schema, row_data)?;
                    let right_val =
                        self.evaluate_expression_with_row(right, table_schema, row_data)?;
                    self.compare_values(&left_val, &right_val, op)
                }
                _ => Err(RengoError::Query {
                    message: format!("Operator {:?} not yet supported in WHERE clause", op),
                }),
            },
            Expression::IsNull { expr, negated } => {
                let val = self.evaluate_expression_with_row(expr, table_schema, row_data)?;
                let is_null = val.as_ref().is_empty();
                Ok(if *negated { !is_null } else { is_null })
            }
            Expression::UnaryOp { op: UnaryOperator::Not, expr } => {
                Ok(!self.evaluate_condition(expr, table_schema, row_data)?)
            }
            _ => Err(RengoError::Query {
                message: "Complex WHERE expressions not yet fully supported".to_string(),
            }),
        }
    }

    /// Compare two values using a comparison operator (numeric when both parse as numbers)
    fn compare_values(&self, left: &Value, right: &Value, op: &BinaryOperator) -> Result<bool> {
        let cmp = if let (Ok(l), Ok(r)) = (
            std::str::from_utf8(left.as_ref()).ok().and_then(|s| s.parse::<f64>().ok()).ok_or(()),
            std::str::from_utf8(right.as_ref()).ok().and_then(|s| s.parse::<f64>().ok()).ok_or(()),
        ) {
            l.partial_cmp(&r).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            left.as_ref().cmp(right.as_ref())
        };

        Ok(match op {
            BinaryOperator::Equal => cmp == std::cmp::Ordering::Equal,
            BinaryOperator::NotEqual => cmp != std::cmp::Ordering::Equal,
            BinaryOperator::LessThan => cmp == std::cmp::Ordering::Less,
            BinaryOperator::LessThanOrEqual => cmp != std::cmp::Ordering::Greater,
            BinaryOperator::GreaterThan => cmp == std::cmp::Ordering::Greater,
            BinaryOperator::GreaterThanOrEqual => cmp != std::cmp::Ordering::Less,
            _ => {
                return Err(RengoError::Query {
                    message: format!("Operator {:?} is not a comparison", op),
                })
            }
        })
    }

    /// Evaluate an expression with access to row data
    fn evaluate_expression_with_row(
        &self,
        expr: &Expression,
        table_schema: &TableSchema,
        row_data: &[Value],
    ) -> Result<Value> {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Null => Ok(Value::new(b"")),
                Literal::Boolean(b) => Ok(Value::from_string(if *b { "true" } else { "false" })),
                Literal::Integer(i) => Ok(Value::from_string(&i.to_string())),
                Literal::Float(f) => Ok(Value::from_string(&f.to_string())),
                Literal::String(s) => Ok(Value::new(s.as_bytes())),
                Literal::Binary(b) => Ok(Value::new(b)),
            },
            Expression::Column(col_name) => {
                let column_index =
                    table_schema.columns.iter().position(|c| c.name == *col_name).ok_or_else(
                        || RengoError::Query {
                            message: format!("Column '{}' not found", col_name),
                        },
                    )?;

                if column_index < row_data.len() {
                    Ok(row_data[column_index].clone())
                } else {
                    Ok(Value::new(b""))
                }
            }
            Expression::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression_with_row(left, table_schema, row_data)?;
                let right_val = self.evaluate_expression_with_row(right, table_schema, row_data)?;

                // Simplified arithmetic operations
                match op {
                    BinaryOperator::Add
                    | BinaryOperator::Subtract
                    | BinaryOperator::Multiply
                    | BinaryOperator::Divide => {
                        // For simplicity, treat as integer operations
                        if left_val.as_ref().len() == 8 && right_val.as_ref().len() == 8 {
                            let left_int =
                                i64::from_le_bytes(left_val.as_ref().try_into().unwrap_or([0; 8]));
                            let right_int =
                                i64::from_le_bytes(right_val.as_ref().try_into().unwrap_or([0; 8]));

                            let result = match op {
                                BinaryOperator::Add => left_int + right_int,
                                BinaryOperator::Subtract => left_int - right_int,
                                BinaryOperator::Multiply => left_int * right_int,
                                BinaryOperator::Divide => {
                                    if right_int == 0 {
                                        return Err(RengoError::Query {
                                            message: "Division by zero".to_string(),
                                        });
                                    }
                                    left_int / right_int
                                }
                                _ => unreachable!(),
                            };

                            Ok(Value::new(&result.to_le_bytes()))
                        } else {
                            Err(RengoError::Query {
                                message: "Arithmetic operations require integer operands"
                                    .to_string(),
                            })
                        }
                    }
                    _ => Err(RengoError::Query {
                        message: "Complex expressions not yet fully supported".to_string(),
                    }),
                }
            }
            _ => Err(RengoError::Query {
                message: "Expression type not yet supported in execution".to_string(),
            }),
        }
    }

    /// Convert value to boolean
    fn value_to_bool(&self, value: &Value) -> Result<bool> {
        if value.as_ref().is_empty() {
            Ok(false)
        } else if value.as_ref().len() == 1 {
            Ok(value.as_ref()[0] != 0)
        } else {
            Ok(true)
        }
    }

    /// Evaluate an expression to a value (simplified)
    fn evaluate_expression(&self, expr: &Expression) -> Result<Value> {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Null => Ok(Value::new(b"")),
                Literal::Boolean(b) => Ok(Value::from_string(if *b { "true" } else { "false" })),
                Literal::Integer(i) => Ok(Value::from_string(&i.to_string())),
                Literal::Float(f) => Ok(Value::from_string(&f.to_string())),
                Literal::String(s) => Ok(Value::new(s.as_bytes())),
                Literal::Binary(b) => Ok(Value::new(b)),
            },
            _ => Err(RengoError::Query {
                message: "Complex expressions not yet supported".to_string(),
            }),
        }
    }

    /// Extract key from WHERE clause (simplified for point queries)
    fn extract_key_from_where(&self, where_clause: &Expression, table_name: &str) -> Result<Key> {
        // For simplicity, assume WHERE clause is "id = value"
        match where_clause {
            Expression::BinaryOp {
                left: _left,
                op: BinaryOperator::Equal,
                right,
            } => {
                // Extract the value (simplified)
                if let Expression::Literal(Literal::Integer(id)) = right.as_ref() {
                    Ok(Key::new(format!("{}:{}", table_name, id).as_bytes()))
                } else {
                    Err(RengoError::Query {
                        message: "Point queries currently support integer IDs only".to_string(),
                    })
                }
            }
            _ => Err(RengoError::Query {
                message: "Point queries require equality condition".to_string(),
            }),
        }
    }

    /// Serialize row data (simplified)
    fn serialize_row(&self, row_data: &[Value]) -> Result<Vec<u8>> {
        // Simple serialization - concatenate lengths and data
        let mut result = Vec::new();
        result.extend_from_slice(&(row_data.len() as u32).to_le_bytes());

        for value in row_data {
            let data = value.as_ref();
            result.extend_from_slice(&(data.len() as u32).to_le_bytes());
            result.extend_from_slice(data);
        }

        Ok(result)
    }

    /// Deserialize row data (simplified)
    fn deserialize_row(&self, data: &[u8]) -> Result<Vec<Value>> {
        let mut offset = 0;
        let mut result = Vec::new();

        // Read number of columns
        if data.len() < 4 {
            return Err(RengoError::Query {
                message: "Invalid row data format".to_string(),
            });
        }

        let column_count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        offset += 4;

        // Read each column value
        for _ in 0..column_count {
            if offset + 4 > data.len() {
                return Err(RengoError::Query {
                    message: "Invalid row data format".to_string(),
                });
            }

            let value_len = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + value_len > data.len() {
                return Err(RengoError::Query {
                    message: "Invalid row data format".to_string(),
                });
            }

            let value_data = &data[offset..offset + value_len];
            result.push(Value::new(value_data));
            offset += value_len;
        }

        Ok(result)
    }

    /// Apply column selection
    fn apply_column_selection(
        &self,
        select_items: &[SelectItem],
        table_schema: &TableSchema,
        row_data: &[Value],
    ) -> Result<(Vec<String>, Vec<Value>)> {
        let mut selected_columns = Vec::new();
        let mut selected_data = Vec::new();

        for item in select_items {
            match item {
                SelectItem::Wildcard => {
                    // Select all columns
                    for (i, column) in table_schema.columns.iter().enumerate() {
                        selected_columns.push(column.name.clone());
                        if i < row_data.len() {
                            selected_data.push(row_data[i].clone());
                        } else {
                            selected_data.push(Value::new(b""));
                        }
                    }
                }
                SelectItem::Expression { expr, alias } => {
                    // For now, only support column references
                    if let Expression::Column(column_name) = expr {
                        let column_index = table_schema
                            .columns
                            .iter()
                            .position(|c| c.name == *column_name)
                            .ok_or_else(|| RengoError::Query {
                                message: format!("Column '{}' not found", column_name),
                            })?;

                        let display_name = alias.as_ref().unwrap_or(column_name).clone();
                        selected_columns.push(display_name);

                        if column_index < row_data.len() {
                            selected_data.push(row_data[column_index].clone());
                        } else {
                            selected_data.push(Value::new(b""));
                        }
                    } else {
                        return Err(RengoError::Query {
                            message: "Complex expressions in SELECT not yet supported".to_string(),
                        });
                    }
                }
            }
        }

        Ok((selected_columns, selected_data))
    }

    /// Get selected column names
    fn get_selected_column_names(
        &self,
        select_items: &[SelectItem],
        table_schema: &TableSchema,
    ) -> Result<Vec<String>> {
        let mut column_names = Vec::new();

        for item in select_items {
            match item {
                SelectItem::Wildcard => {
                    for column in &table_schema.columns {
                        column_names.push(column.name.clone());
                    }
                }
                SelectItem::Expression { expr, alias } => {
                    if let Expression::Column(column_name) = expr {
                        let display_name = alias.as_ref().unwrap_or(column_name).clone();
                        column_names.push(display_name);
                    } else {
                        return Err(RengoError::Query {
                            message: "Complex expressions in SELECT not yet supported".to_string(),
                        });
                    }
                }
            }
        }

        Ok(column_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rengo_storage::MemoryEngine;

    #[tokio::test]
    async fn test_create_table() {
        let storage = Box::new(MemoryEngine::new(&Default::default()).await.unwrap());
        let mvcc = MVCCStorage::new();
        let catalog = SchemaCatalog::new();

        let context = ExecutionContext { storage, mvcc, catalog };

        let mut executor = QueryExecutor::new(context);

        let stmt = CreateTableStatement {
            if_not_exists: false,
            name: "users".to_string(),
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    nullable: false,
                    default: None,
                    auto_increment: true,
                },
                ColumnDef {
                    name: "name".to_string(),
                    data_type: DataType::Varchar { length: Some(255) },
                    nullable: true,
                    default: None,
                    auto_increment: false,
                },
            ],
            constraints: vec![],
        };

        let result = executor.execute(Statement::CreateTable(stmt)).await.unwrap();
        assert_eq!(result.affected_rows, 1);

        // Check that table was added to catalog
        assert!(executor.context.catalog.get_table("users").is_some());
    }

    #[tokio::test]
    async fn test_insert_select() {
        let storage = Box::new(MemoryEngine::new(&Default::default()).await.unwrap());
        let mvcc = MVCCStorage::new();
        let mut catalog = SchemaCatalog::new();

        // Add table schema
        let table_schema = TableSchema {
            name: "users".to_string(),
            columns: vec![
                ColumnSchema {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    nullable: false,
                    default_value: None,
                    auto_increment: true,
                },
                ColumnSchema {
                    name: "name".to_string(),
                    data_type: DataType::Varchar { length: Some(255) },
                    nullable: true,
                    default_value: None,
                    auto_increment: false,
                },
            ],
            primary_key: None,
            indexes: vec![],
        };
        catalog.add_table(table_schema);

        let context = ExecutionContext { storage, mvcc, catalog };

        let mut executor = QueryExecutor::new(context);

        // Insert a row
        let insert_stmt = InsertStatement {
            table_name: "users".to_string(),
            columns: None,
            values: vec![vec![
                Expression::Literal(Literal::Integer(1)),
                Expression::Literal(Literal::String("Alice".to_string())),
            ]],
        };

        let result = executor.execute(Statement::Insert(insert_stmt)).await.unwrap();
        assert_eq!(result.affected_rows, 1);

        // Select the row back
        let select_stmt = SelectStatement {
            distinct: false,
            columns: vec![SelectItem::Wildcard],
            from: Some("users".to_string()),
            where_clause: Some(Expression::BinaryOp {
                left: Box::new(Expression::Column("id".to_string())),
                op: BinaryOperator::Equal,
                right: Box::new(Expression::Literal(Literal::Integer(1))),
            }),
            group_by: vec![],
            having: None,
            order_by: vec![],
            limit: None,
            offset: None,
        };

        let result = executor.execute(Statement::Select(select_stmt)).await.unwrap();
        assert_eq!(result.affected_rows, 1);
        assert_eq!(result.columns.len(), 2);
        assert_eq!(result.rows.len(), 1);
    }
}
