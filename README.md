## Lava
[Sample-output](https://github.com/Harshit933/lava/blob/master/sample-output)

**To run**

(With docker)
1. `docker compose build`
2. `docker compose up`
3. Then in other terminal make the CURL request as: `curl http://0.0.0.0:3000/test`

(With Rust)
1. Modify the `.env` file to point to the path of the `loans-cli`.
2. `cargo r`
3. Then in other terminal make the CURL request as: `curl http://0.0.0.0:3000/test`

**TODO:**

I have yet to implement the function to store the contract-ids. But I guess it should be fairly easy to do.

**Future TODOs that I think would be beneficial:**
1. Implement a certain request where the `mnemonics` are passed and then run test bases on those `mnemonic` rather than generating them fresh every time.
