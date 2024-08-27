import { hash, shortString } from "https://esm.run/starknet@5.14";
// import { hash, shortString } from "starknet";


const AssertionMadeSelector = "0x02b74480ce203d4ca6ee46c062a2d658129587cdccb3904c52a6f4dfb406a3f2";

const AssertionDisputedSelector = "0x033efc8167fe91406d16c1680e593c3c2cad864220645cc2a48efd67e8f7ca73"

const AssertionSettledSelector = "0x00ed635610f8859765fb89b19bec7866c02bbc2f03bb048acec6ba6536aa7cb9";

const ContractAddress = "0x04d559e9a6bedc3cea5cf87961e24a87340699ff793ad1c6e3349fb8d6a8e91f"


const filter = {
  // Only request header if any event matches.
  header: {
    weak: true,
  },
  events: [
    {
      fromAddress:
      ContractAddress,
      keys: [AssertionMadeSelector],
      includeTransaction: true,
      includeReceipt: false,
    },
    {
      fromAddress:
      ContractAddress,
      keys: [AssertionSettledSelector],
      includeTransaction: true,
      includeReceipt: false,
    },
    {
      fromAddress:
      ContractAddress,
      keys: [AssertionDisputedSelector],
      includeTransaction: true,
      includeReceipt: false,
    },
  ],
};

function escapeInvalidCharacters(str) {
  return str.replace(/^[\x00-\x1F]+/, "");
}

function trimLeadingZeros(hexString) {
  // Check if the string starts with '0x'
  if (hexString.startsWith('0x')) {
    // Remove '0x', trim zeros, then add '0x' back
    const trimmed = hexString.slice(2).replace(/^0+/, '');
    return '0x' + trimmed;
  }
  // If it doesn't start with '0x', just trim zeros
  return hexString.replace(/^0+/, '');
}

function combineU256(low, high) {
  // Convert hex strings to BigInt
  const lowBigInt = BigInt(low);
  const highBigInt = BigInt(high);
  
  // Combine the low and high parts
  return (highBigInt << BigInt(128)) + lowBigInt;
}


function concatenateHexStrings(hexArray) {
  return hexArray.map(str => str.replace(/^0x/, '')).join('');
}

function decodeTransfersInBlock({ header, events }) {
  const { blockNumber, blockHash, timestamp } = header;
  return events.flatMap(({ event, transaction }) => {
    if (event.keys[0] == AssertionMadeSelector) {
    const transactionHash = transaction.meta.hash;
    const dataId = `${transactionHash}_${event.index ?? 0}`;

    // Parse the claim data dynamically
    const claimLength = parseInt(event.data[3], 16); // Convert hex to integer
    const claimData = event.data.slice(4, 4 + claimLength+1);
    const remainingData = event.data.slice(6 + claimLength);
    claimData[claimData.length - 1] = trimLeadingZeros(claimData[claimData.length - 1]);
    const [
      assertionId,
      domainIdLow,
      domainIdHigh,
      ,  // Skip the claim length as we've already used it
      ...rest
    ] = event.data;
    
    const [
      asserter,
      callbackRecipient,
      escalationManager,
      caller,
      expirationTimestamp,
      currency,
      bondLow,
      bondHigh,
      identifier
    ] = remainingData;
    
    const bondBigInt = combineU256(bondLow, bondHigh);
    const bond = bondBigInt.toString(); // Convert to string for database storage    
    
    const domainIdBigInt = combineU256(domainIdLow, domainIdHigh);
    const domainId = domainIdBigInt.toString(); // Convert to string for database storage  
    return {
      insert: {
      network: "starknet-sepolia",
      data_id: dataId,
      assertion_id: assertionId.toString(),
      domain_id: domainId,
      claim:concatenateHexStrings(claimData), // Store claim as a JSON string
      asserter: asserter,
      callback_recipient: callbackRecipient,
      escalation_manager: escalationManager,
      caller: caller,
      expiration_timestamp: new Date(Number(expirationTimestamp) * 1000).toISOString(),
      currency: currency,
      bond:  bond,
      identifier: trimLeadingZeros(identifier), 
      updated_at: timestamp,
      updated_at_tx: transaction.meta.hash,
      },
    };
  } else if(event.keys[0] == AssertionDisputedSelector) {
      // Update request entity.
      const assertionId = event.data[0];
      const caller = event.data[1];
      const disputer = event.data[2];
      const request_id = event.data[3];
      return {
        entity: {
          assertion_id: assertionId,
        },
        update: {
          disputer_caller: caller,
          disputer: disputer,
          disputed: true,
          dispute_id: request_id,
        },
      };
    } else if (event.keys[0] == AssertionSettledSelector){
      const assertionId = event.data[0];
      const bondRecipient = event.data[1];
      const disputed = event.data[2]=='0x0000000000000000000000000000000000000000000000000000000000000000' ? false: true;
      const settlementResolution = event.data[3]=='0x0000000000000000000000000000000000000000000000000000000000000000' ? false: true;
      const settleCaller = event.data[4];
      return {
        entity: {
          assertion_id: assertionId,
        },
        update: {
          settlement_resolution: settlementResolution, 
          disputed: disputed, 
          settle_caller: settleCaller,
          settled: true,
          updated_at: timestamp,
          updated_at_tx: transaction.meta.hash,
        },
      };
    } else {
      return [];
    }
  });
}

// Configure indexer for streaming Starknet Goerli data starting at the specified block.
export const config = {
  streamUrl: "https://sepolia.starknet.a5a.ch",
  startingBlock: Number(Deno.env.get("STARTING_BLOCK") || 86000),
  network: "starknet",
  filter,
  batchSize: 1,
  finality: "DATA_STATUS_PENDING",
  sinkType: "postgres",
  sinkOptions: {
    // Send data as returned by `transform`.
    entityMode: true,
    schema: "public",
    upsertConflictFields: ["assertion_id"],
    updateOnConflict: true,
  },
};

// Transform each block using the function defined in starknet.js.
export default decodeTransfersInBlock;

