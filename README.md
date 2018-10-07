# Pig 🐷

This tool was created to simplify working with PostgreSQL for every day db tasks for personal projects. Common tasks like creating simple migrations, applying them incrementally on multiple servers, and seeing what's in a database should be easy to do.

# What problem does this solve?

Most of the time when working with SQL databases for personal projects, i'm only doing a few things. Adding tables, adding columns, and applying them to one or more servers. These changes can been seen as a date ordered sequence of SQL files
to push up to a database. Pig stores what latest file has been applied on your db so next time you apply the migrations, it only uploads the newest ones. I added a few helper utilities for quickly adding common SQL commands to your current migration (create table, add column, drop table). You may face other challenges with migrations in real world projects, but this tool might offer you a quick way to get going with SQL and having fun.

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
