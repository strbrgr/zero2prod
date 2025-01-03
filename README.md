Working on the book Zero 2 Production with Axum.

Learned:
### `error: error with configuration: relative URL without a base` when running `sqlx migrate run`
This happened at the end of chapter 5 when we tried to migrate the new database URL. I forgot to wrap that URL in double quotes.
