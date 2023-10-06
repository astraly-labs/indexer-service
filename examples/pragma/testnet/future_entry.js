import { hash, shortString } from "https://esm.run/starknet@5.14";

const filter = {
  // Only request header if any event matches.
  header: {
    weak: true,
  },
  events: [
    {
      fromAddress:
        "0x620a609f88f612eb5773a6f4084f7b33be06a6fed7943445aebce80d6a146ba",
      keys: [hash.getSelectorFromName("SubmittedFutureEntry")],
    },
  ],
};

function decodeEventsInBlock({ header, events }) {
  const { blockNumber, blockHash, timestamp } = header;
  return events.map(({ event, receipt }) => {
    const { transactionHash } = receipt;
    const dataId = `${transactionHash}_${event.index}`;

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
    const publisherName = shortString.decodeShortString(publisher);
    const sourceName = shortString.decodeShortString(source);
    const pairIdName = shortString.decodeShortString(pairId);

    // Convert to snake_case because it works better with postgres.
    return {
      network: "starknet-goerli",
      pair_id: pairIdName,
      data_id: dataId,
      block_hash: blockHash,
      block_number: +blockNumber,
      block_timestamp: timestamp,
      transaction_hash: transactionHash,
      price: +price,
      timestamp: entryTimestamp,
      publisher: publisherName,
      source: sourceName,
      volume: +volume,
      expiration_timestamp: expirationTimestamp,
    };
  });
}

// Configure indexer for streaming Starknet Goerli data starting at the specified block.
export const config = {
  streamUrl: "https://mainnet.starknet.a5a.ch",
  startingBlock: 865000,
  network: "starknet",
  filter,
  sinkType: "postgres",
  sinkOptions: {
    // Send data as returned by `transform`.
    // When `raw = false`, the data is sent together with the starting and end cursor.
    raw: true,
  },
};

// Transform each block using the function defined in starknet.js.
export default decodeEventsInBlock;
