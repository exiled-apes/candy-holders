import { program } from 'commander'
import Arweave from 'arweave';
import { ArweaveSigner, bundleAndSignData, createData } from "arbundles";

import fs from 'fs';

program
    .version('0.0.1')
    .option('-d, --directory <name>', 'path to directory to upload')
    .option('-k, --key-path <key-path>', 'path to key file')
    .parse()

const arweave = Arweave.init({
    host: 'arweave.net',
    port: 443,
    protocol: 'https'
});

const main = async () => {
    const { directory, keyPath } = program.opts()

    const jwk = JSON.parse(fs.readFileSync(keyPath, { encoding: 'utf8' }))

    const signer = new ArweaveSigner(jwk);

    const files = fs.readdirSync(directory);

    for (const file of files) {
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
        const new_url = `https://arweave.net/${response.data.id}`

        console.log(metadata_address, ' => ', new_url) // TODO store this in sqlite
    }
}
main();