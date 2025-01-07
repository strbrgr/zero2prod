Working on the book Zero 2 Production with Axum.

## Progress
- [x] Chapter 1
- [x] Chapter 2
- [x] Chapter 3
- [x] Chapter 4
- [x] Chapter 5
- [] Chapter 6

## Learned
### Different status code returned
Axum returns `StatusCode::UNPROCESSABLE_ENTITY` whereas actix-web `StatusCode::BAD_REQUEST` when a handler can't process the body of an incoming request.

### `error: error with configuration: relative URL without a base` when running `sqlx migrate run`
This happened at the end of chapter 5 when we tried to migrate the new database URL. I forgot to wrap that URL in double quotes.
