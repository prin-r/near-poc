const axios = require('axios');
const BN = require('bn.js');
const nearAPI = require("near-api-js");

// Path to keys
// /Users/mumu/.near-credentials/
// └── default
//     ├── 1.mumu.testnet.json
//     ├── 2.mumu.testnet.json
//     └── mumu.testnet.json
const pathToNearKeys = "/Users/mumu/.near-credentials/";
const nearConfig = {
    sender: "mumu.testnet",
    networkId: 'default',
    nodeUrl: 'https://rpc.testnet.near.org',
    contractName: '1.mumu.testnet',
    methodName: 'relay',
    walletUrl: 'https://wallet.testnet.near.org',
    helperUrl: 'https://helper.testnet.near.org'
}

const BAND_ENDPOINT = "https://poa-api.bandchain.org/oracle/request_prices";
const E9 = '1000000000';
const symbols = ["BTC","ETH"];

const sleep = async ms => new Promise(r => setTimeout(r, ms));

const getConnectedAccount = async () => {
    // Create key store
    const keyStore = new nearAPI.keyStores.UnencryptedFileSystemKeyStore(pathToNearKeys);

    // Initializing connection to the NEAR node.
    const near = await nearAPI.connect({ deps: { keyStore }, ...nearConfig });

    return await near.account(nearConfig.sender);
}

const getPricesFromBand = async () => {
    const rawResults = await axios.post(BAND_ENDPOINT, {symbols, min_count:3, ask_count:4}).then(r => r.data['result']);

    const result = {
        symbols: [],
        rates:[],
        resolve_times:[],
        request_ids:[]
    }

    for ({symbol,multiplier,px,request_id,resolve_time} of rawResults) {
        if (multiplier !== E9) {
            throw "multiplier is not equal 1_000_000_000";
        }
        result.symbols.push(symbol);
        result.rates.push(Number(px));
        result.resolve_times.push(Number(resolve_time + E9.slice(1)));
        result.request_ids.push(Number(request_id));
    }

    return result
}

(async () => {
    const account = await getConnectedAccount();

    while (true) {
        try {
            console.log("Getting prices from BAND ...")
            const prices = await getPricesFromBand();
            console.log(prices);

            console.log("Sending relay to NEAR ...")
            const functionCallResponse = await account.functionCall(
                nearConfig.contractName,
                nearConfig.methodName,
                prices,
                new BN("10000000000000"),
                new BN("0")
            );

            console.log("status: ", functionCallResponse.status);
            console.log(functionCallResponse.transaction);
        } catch (e) {
            console.log(e)
        }

        let count = 10;
        while (count > 0) {
            process.stdout.clearLine();
            process.stdout.cursorTo(0);
            process.stdout.write('countdown: ' + count);
            await sleep(1000);
            count--;
        }
        console.log("\n=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=")
    }
})();
