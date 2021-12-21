import { Connection, PublicKey } from "@solana/web3.js"
import { createInterface } from 'readline';
import { program } from 'commander'
import pRetry from 'p-retry';

program
    .version('0.0.1')
    .option('-e, --rpc-host <string>', 'rpc host', 'https://api.mainnet-beta.solana.com')
    .option('-c, --chill <number>', 'sleep per token (please be nice to free rpc servers) ', '100')
    .parse()

const { rpcHost, chill } = program.opts()
const connection = new Connection(rpcHost, 'singleGossip')

async function sleep(millis: number) {
    return new Promise(resolve => setTimeout(resolve, millis));
}

async function mineCurrentHolder(tokenAccount: string): Promise<string | undefined> {
    try {
        const largestAccounts = await connection.getTokenLargestAccounts(new PublicKey(tokenAccount))
        const largestPDA = largestAccounts.value.shift()

        if (largestPDA && largestPDA.address) {
            const largestWallet = await connection.getParsedAccountInfo(largestPDA?.address!);
            const data = largestWallet.value?.data.valueOf();

            //@ts-ignore
            return data?.parsed?.info?.owner;
        }
    } catch (err) {
        console.error(err)
    }
}

async function main() {
    const rest = parseInt(chill, 10)

    const lineReader = createInterface({
        input: process.stdin,
        crlfDelay: Infinity
    });

    for await (const line of lineReader) {
        const tokenAccount = line.split(' ').pop()!
        const currentHolder = await pRetry(async () => await mineCurrentHolder(tokenAccount), {
            onFailedAttempt: (err) => console.error(`mining ${tokenAccount} failed.`, err),
            retries: 4,
        })
        if (currentHolder) {
            console.log(`${currentHolder}, ${tokenAccount}`)
        }

        await sleep(rest)
    }
}

(async () => await main())();