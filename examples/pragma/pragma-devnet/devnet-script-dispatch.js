import { hash } from "https://esm.run/starknet@5.14";

const HYPERLANE_MAILBOX_CONTRACT =
  "0x064bb5e29a7e9fc67dedabb6d1c385050feef028563078daa9025dd8b218e596";
const FEED_ID_SIZE = 64;

const filter = {
  // Only request header if any event matches.
  header: {
    weak: true,
  },
  events: [
    {
      fromAddress: HYPERLANE_MAILBOX_CONTRACT,
      keys: [hash.getSelectorFromName("Dispatch")],
      includeTransaction: true,
      includeReceipt: false,
    },
  ],
};

function decodeFeedId(feedIdHex) {
  const feedId = BigInt(`0x${feedIdHex}`);
  const assetClass = Number((feedId >> BigInt(232)) & BigInt(0xffff));
  const feedType = Number((feedId >> BigInt(216)) & BigInt(0xffff));
  const pairId = feedId & BigInt((1n << 216n) - 1n);

  return { assetClass, feedType, pairId };
}

function getFeedSize(assetClass, feedType) {
  const mainType = feedType >> 8;
  switch (assetClass) {
    case 0: // Crypto
      switch (mainType) {
        case 0: // Unique
          return 214; // 856 bits / 4 = 214 hex characters
        case 1: // Twap
          return 470; // 1880 bits / 4 = 470 hex characters
        default:
          throw new Error(`Unknown feed type: ${feedType}`);
      }
    default:
      throw new Error(`Unknown asset class: ${assetClass}`);
  }
}

function decodeFeedsUpdatedFromHyperlaneMessage(hexData) {
  let data = hexData.map((hex) => {
    let trimmed = hex.replace(/^0x/, "");
    trimmed = trimmed.slice(32);
    return trimmed === "" ? "" : trimmed;
  });
  data = data.join("");
  console.log(data);

  const numberOfUpdates = Number(data.slice(0, 4));
  data = data.slice(4);

  const feedIdsUpdated = [];
  for (let i = 0; i < numberOfUpdates; i++) {
    const feedIdHex = data.slice(0, FEED_ID_SIZE);
    const { assetClass, feedType } = decodeFeedId(feedIdHex);
    feedIdsUpdated.push(`0x${feedIdHex}`);
    data = data.slice(getFeedSize(assetClass, feedType));
  }
  return feedIdsUpdated;
}

export function decodeTransfersInBlock({ header, events }) {
  const { blockNumber, blockHash, timestamp } = header;

  return events.flatMap(({ event, transaction }) => {
    const transactionHash = transaction.meta.hash;
    const hyperlaneMessageNonce = parseInt(event.data[6], 16);
    const messageBody = event.data.slice(15);
    console.log(messageBody);
    const feedsUpdated = decodeFeedsUpdatedFromHyperlaneMessage(messageBody);

    return {
      network: "pragma-devnet",
      block_hash: blockHash,
      block_number: +blockNumber,
      block_timestamp: timestamp,
      transaction_hash: transactionHash,
      hyperlane_message_nonce: hyperlaneMessageNonce,
      feeds_updated: feedsUpdated,
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
