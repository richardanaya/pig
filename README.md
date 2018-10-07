# Pig 🐷

This tool was created to simplify working with PostgreSQL for every day development tasks. Common tasks like creating simple migrations, applying them incrementally on multiple servers, and seeing what's in a database should be easy to do.

# Example Usage

```bash
export PIG_CONNECTION_STRING="<your secret connection string>"

# Create a migration
pig create "My first migration"
# Add SQL to apply/revert people table to latest migration
pig modify create-table people     
# Add SQL to apply/revert name column to latest migration
pig modify add-column people name TEXT
# See whats going to be applied
pig plan                        
# Apply migrations in current directory
pig apply    
# See people table on db
pig show tables
# See people table's columns
pig show table people                       
# Should see nothing to apply
pig plan  
# Create new migration for dropping people
pig create "Drop people"       
# Add drop table command
pig modify drop-table people  
# Should only see one migration to apply
pig plan   
# Apply only the newest migration
pig apply
# Now people table is removed from db
pig show tables
```
