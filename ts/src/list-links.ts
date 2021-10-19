import { program } from 'commander'
const sqlite3 = require('sqlite3').verbose();

import fs from 'fs';

program
    .version('0.0.1')
    .option('-db, --db-path <path>', 'path to sqlite db')
    .parse()

const main = async () => {
    const { dbPath } = program.opts()
    let db = new sqlite3.Database(dbPath);

    db.each("SELECT * FROM metadatas ORDER BY token_address", function (err: any, metadata: any) {
        if (err) {
            console.error('metadata query', err);
            return
        }

        db.get("SELECT * FROM repairs WHERE token_address = ?", metadata.token_address, function (err: any, repair: any) {
            if (err) {
                console.error('repairs query', err);
                return
            }

            if (!repair) {
                console.log(metadata.uri)
            } else {
                console.log(repair.new_uri)
            }
        });
    });


    await db.close() // todo move outside of loop and delete return
}


main();