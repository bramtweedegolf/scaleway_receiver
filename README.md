How to deploy
-------------

1. Increase the version number.
2. Run 'cargo build' to check if it compiles.
3. Create a zip file from src/ and Cargo.lock and Cargo.toml.
4. Upload the zip in the scaleway console.

How to test (on production)
---------------------------

1. Send a request: curl --data "{test}"  https://wingutestfunctionsqpm70nrs-wingu-rust-receiver.functions.fnc.nl-ams.scw.cloud
2. Check logs here: https://6651b1da-cee5-44f0-b6ea-f0cd9271c857.dashboard.cockpit.fr-par.scw.cloud/d/scw-serverless-functions-logs/serverless-functions-logs?orgId=1&var-function_name=wingutestfunctionsqpm70nrs-wingu-rust-receiver&var-datasource=af7100a0-5915-413f-9951-ee4d93acf5e7
