import { hash, shortString } from "https://esm.run/starknet@5.14";

const filter = {
  // Only request header if any event matches.
  header: {
    weak: true,
  },
  events: [
    {
      fromAddress:
        "0x60240f2bccef7e64f920eec05c5bfffbc48c6ceaa4efca8748772b60cbafc3",
      keys: [hash.getSelectorFromName("SubmittedSpotEntry")],
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

    const [entryTimestamp, source, publisher, price, pairId, volume] =
      event.data;

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
    };
  });
}

// Configure indexer for streaming PragmaGix data starting at the specified block.
export const config = {
  streamUrl: "https://devnet.pragma.a5a.ch",
  startingBlock: Number(0),
  network: "starknet",
  filter,
  batchSize: 1,
  finality: "DATA_STATUS_PENDING",
  sinkType: "postgres",
  sinkOptions: {
    // Send data as returned by `transform`.
    // When `raw = false`, the data is sent together with the starting and end cursor.
    raw: true,
  },
};

// Transform each block using the function defined in starknet.js.
export default decodeTransfersInBlock;
