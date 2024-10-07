import { hash, shortString } from "https://esm.run/starknet@5.14";

const filter = {
  // Only request header if any event matches.
  header: {
    weak: true,
  },
  events: [
    {
      fromAddress:
        "0x39986b4fdb2b29e81bcfab5640c614797d75116bca67f0b7c4690c19a3392bc",
      keys: [hash.getSelectorFromName("SubmittedFutureEntry")],
      includeTransaction: true,
      includeReceipt: false,
    },
  ],
};

function escapeInvalidCharacters(str) {
  return str.replace(/^[\x00-\x1F]+/, "");
}

function decodeTransfersInBlock({ header, events }) {
  const { blockNumber, blockHash, timestamp } = header;
  return events.flatMap(({ event, transaction }) => {
    const transactionHash = transaction.meta.hash;

    const dataId = `${transactionHash}_${event.index ?? 0}`;

    const [
      entryTimestamp,
      source,
      publisher,
      price,
      pairId,
      volume,
      expirationTimestamp,
    ] = event.data;

    // Convert felts to string
    const publisherName = escapeInvalidCharacters(
      shortString.decodeShortString(publisher),
    );
    const sourceName = escapeInvalidCharacters(
      shortString.decodeShortString(source),
    );
    const pairIdName = escapeInvalidCharacters(
      shortString.decodeShortString(pairId),
    );

    // Convert to snake_case because it works better with postgres.
    return {
      network: "pragma-devnet",
      pair_id: pairIdName,
      data_id: dataId,
      block_hash: blockHash,
      block_number: +blockNumber,
      block_timestamp: timestamp,
      transaction_hash: transactionHash,
      price: +price,
      timestamp: new Date(Number(entryTimestamp) * 1000).toISOString(),
      publisher: publisherName,
      source: sourceName,
      volume: +volume,
      expiration_timestamp: new Date(Number(expirationTimestamp)).toISOString(),
    };
  });
}

// Configure indexer for streaming Starknet Goerli data starting at the specified block.
export const config = {
  streamUrl: "https://devnet.pragma.a5a.ch",
  startingBlock: Number(0),
  network: "pragma-devnet",
  filter,
  sinkType: "postgres",
  finality: "DATA_STATUS_PENDING",
  batchSize: 1,
  sinkOptions: {
    // Send data as returned by `transform`.
    // When `raw = false`, the data is sent together with the starting and end cursor.
    raw: true,
  },
};

// Transform each block using the function defined in starknet.js.
export default decodeTransfersInBlock;
