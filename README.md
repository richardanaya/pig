# Pig

This tool was created to simplify working with PostgreSQL for every day development tasks. Common tasks like creating simple migrations, applying/reverting them, and seeing whats in a database should be easy to do.

# Example Usage

```bash
export PIG_CONNECTION_STRING="<your secret connection string>"

# Create a migration
pig create "My first migration"
# Generates code to apply/revert people table
pig modify add-table people     
# Generates code to apply/revert name column
pig modify add-column name TEXT
# See whats going to be applied
pig plan                        
# Apply migrations in current directory
pig apply                       
# Should see nothing to apply
pig plan                        
# Rollback last migration
pig rollback                    
```
