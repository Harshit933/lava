## Lava

**To run**

(With docker)
1. `docker compose build`
2. `docker compose up`
3. Then in other terminal make the CURL request as: `curl http://0.0.0.0:3000/test`

(With Rust)
1. Modify the `.env` file to point to the path of the `loans-cli`.
2. `cargo r`
3. Then in other terminal make the CURL request as: `curl http://0.0.0.0:3000/test`

There are many things where the code can be cleaned
1. I am waiting for sometime after each request I am doing to the testnet servers.
2. The `error.rs` file needs to be built, I have skipped that.
3. The `fn test()` could be broken down into smaller functions.
4. For now all the functions are written without using any type of design pattern like the most common one would be to use the builder pattern for rust.

**TODO:**

I have yet to implement the function to store the contract-ids. But I guess it should be fairly easy to do.

**Future TODOs that I think would be beneficial:**
1. Implement a certain request where the `mnemonics` are passed and then run test bases on those `mnemonic` rather than generating them fresh every time.
