# Pig 🐷

This tool was created to simplify working with PostgreSQL for every day development tasks. Common tasks like creating simple migrations, applying them incrementally on multiple servers, and seeing what's in a database should be easy to do.

# Example Usage

```bash
export PIG_CONNECTION_STRING="<your secret connection string>"

# Create a migration
pig create "My first migration"
# Add SQL to apply/revert people table to latest migration
pig modify add-table people     
# Add SQL to apply/revert name column to latest migration
pig modify add-column people name TEXT
# See whats going to be applied
pig plan                        
# Apply migrations in current directory
pig apply                       
# Should see nothing to apply
pig plan                 
```
