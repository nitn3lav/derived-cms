# Getting started with the example

1. run the example

   ```sh
   cargo run # or cargo run --release
   ```

2. You should now be able to go to http://localhost:3000/posts and create/edit posts
3. or use the API instead:

   ```console
   ❯ curl http://localhost:3000/api/v1/posts | jq
     % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                    Dload  Upload   Total   Spent    Left  Speed
   100   119  100   119    0     0  52678      0 --:--:-- --:--:-- --:--:-- 59500
   [
     {
       "id": "7daedfb5-ef30-45fd-a80e-67be4f54b506",
       "title": "Test",
       "date": "2026-01-15T14:20:00Z",
       "content": [
           {
               "type": "text",
               "data": "Hello, world"
           }
       ],
       "draft": false
     }
   ]
   ```
