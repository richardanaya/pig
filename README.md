# Pig

This tool was created to simplify working with PostgreSQL for every day development tasks. Common tasks like creating simple migrations, applying/reverting them, and seeing whats in a database should be easy to do.

# Example Usage

```bash
export PIG_CONNECTION_STRING="<your secret connection string>"
pig create "My first migration" # Create a migration
pig modify add-table people     # Generates code to apply/revert people table
pig modify add-column name TEXT # Generates code to apply/revert name column
pig plan                        # See whats going to be applied
pig apply                       # Apply migrations in current directory
pig plan                        # Should see nothing to apply
pig revert                      # Revert
```
