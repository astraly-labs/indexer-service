import { hash, shortString } from "https://esm.run/starknet@5.14";

const filter = {
  // Only request header if any event matches.
  header: {
    weak: true,
  },
  events: [
    {
      fromAddress:
        "0x2a85bd616f912537c50a49a4076db02c00b29b2cdc8a197ce92ed1837fa875b",
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
      network: "starknet-mainnet",
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

// Configure indexer for streaming Starknet Goerli data starting at the specified block.
export const config = {
  streamUrl: "https://pragma-mainnet.starknet.a5a.ch",
  startingBlock: Number(Deno.env.get("STARTING_BLOCK") || 416_000),
  network: "starknet",
  finality: "DATA_STATUS_PENDING",
  filter,
  sinkType: "postgres",
  sinkOptions: {
    // Send data as returned by `transform`.
    // When `raw = false`, the data is sent together with the starting and end cursor.
    raw: true,
  },
};

// Transform each block using the function defined in starknet.js.
export default decodeTransfersInBlock;
