```
cargo run repair schema.rb
```

Outputs SQL statements to "repair" the database schema. For now it just removes the LIMIT 255
from varchar columns.

```
cargo run diff schema1.rb schema2.rb
```

Prints out the differences (missing tables, missing columns, different column options)
between two schemas.
