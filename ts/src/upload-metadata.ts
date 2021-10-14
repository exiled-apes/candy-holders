import { program } from 'commander'
import { ArweaveSigner, createData } from "arbundles";
const sqlite3 = require('sqlite3').verbose();

import fs from 'fs';

program
    .version('0.0.1')
    .option('-d, --directory <path>', 'path to directory to upload')
    .option('-db, --db-path <path>', 'path to sqlite db')
    .option('-k, --key-path <path>', 'path to key file')
    .parse()

const main = async () => {
    const { directory, dbPath, keyPath } = program.opts()

    const jwk = JSON.parse(fs.readFileSync(keyPath, { encoding: 'utf8' }))

    const signer = new ArweaveSigner(jwk);
    let db = new sqlite3.Database(dbPath);

    for (const file of fs.readdirSync(directory)) {
        const data = fs.readFileSync(`${directory}/${file}`);

        const item = createData(data, signer, {
            tags: [{
                name: 'Content-Type',
                value: 'application/json; charset=utf-8',
            }]
        });

        await item.sign(signer);

        const response = await item.sendToBundler();
        if (response.status != 200) {
            console.error(`Sent ${file} bundler with response: ${response.status} / ${response.statusText}`);
            console.error(response.headers);
            console.error(response.data);
            continue;
        }

        const metadata_address = file.split('.json').shift();
        const new_uri = `https://arweave.net/${response.data.id}`

        await db.run("update repairs set new_uri = ? where metadata_address = ?", [
            new_uri, metadata_address,
        ])
    }

    await db.close() // todo move outside of loop and delete return
}


main();