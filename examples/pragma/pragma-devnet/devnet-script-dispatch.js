import { hash, shortString } from "https://esm.run/starknet@5.14";
import * as ethers from "https://esm.run/ethers";

const filter = {
  // Only request header if any event matches.
  header: {
    weak: true,
  },
  events: [
    {
      fromAddress:
        "0x41c20175af14a0bfebfc9ae2f3bda29230a0bceb551844197d9f46faf76d6da",
      keys: [hash.getSelectorFromName("Dispatch")],
      includeTransaction: true,
      includeReceipt: false,
    },
  ],
};

function escapeInvalidCharacters(str) {
  return str.replace(/^[\x00-\x1F]+/, "");
}


// number of udpate u16 (4 char)
// Unique()
// feedID u256
// ts u64
// num source u16
// decimal u8 
// price u256
// volume u256 
// 256 + 64 + 16 + 8 + 256 + 256 = 856 / 4 = 214
function decodeHyperlaneMessageBody(hexData) {
  const uniqueSize = 214;
  let data = hexData.map(hex => {
      // Remove '0x' prefix
      let trimmed = hex.replace(/^0x/, '');
      
      // Remove the first 32 characters
      trimmed = trimmed.slice(32);
      
      // If the string became empty after processing, return an empty string
      return trimmed === '' ? '' : trimmed;
  });
  data = data.join('');
  let numberOfUpdate = Number(data.slice(0,4));
  data = data.slice(4);
  // parse unique
  let feedId = data.slice(0,64);
  let timestamp = data.slice(64, 80);
  let num_source = data.slice(80,84);
  let decimal = data.slice(84,86);
  let price = data.slice(86,86+64);
  let feedTypeId = Number(data.slice(4,5));
  let sizeData = uniqueSize; //TODO conditioné par type de donné

  let res = {
    feedId: feedId,
    feedTypeId: feedTypeId,
    sizeData : sizeData,
    timestamp: timestamp,
    num_source,
    decimal,
    price,
  };
  // concat data 

  console.log(res);
}

function decodeTransfersInBlock({ header, events }) {
  const { blockNumber, blockHash, timestamp } = header;
  
  return events.flatMap(({ event, transaction }) => {
    const transactionHash = transaction.meta.hash;
    
    console.log(event.data);

    const nonce = event.data[6];

    // retrieve body 
    // decode body
    // recuperer tout les feedId a l'interieur
    let messageBody = event.data.slice(15);
    let decoded = decodeHyperlaneMessageBody(messageBody);
    console.log(decoded);
    // Convert to snake_case because it works better with postgres.
    return {
      network: "pragma-devnet",
      block_hash: blockHash,
      block_number: +blockNumber,
      block_timestamp: timestamp,
      transaction_hash: transactionHash,
    };
  });
}

// Configure indexer for streaming PragmaGix data starting at the specified block.
export const config = {
  streamUrl: "https://devnet.pragma.a5a.ch",
  startingBlock: Number(220_840),
  network: "starknet",
  filter,
  batchSize: 1,
  finality: "DATA_STATUS_PENDING",
  sinkType: "console",
  sinkOptions: {
    // Send data as returned by `transform`.
    // When `raw = false`, the data is sent together with the starting and end cursor.
    raw: true,
  },
};

// Transform each block using the function defined in starknet.js.
export default decodeTransfersInBlock;
