/* PostgreSQL Entity mode example
 *
 * This example shows how to use entity mode.
 * It listens for VRF requests and status changes from the Pragma's smart
 * contract on testnet.
 */

const fromAddress =
  "0x4fb09ce7113bbdf568f225bc757a29cb2b72959c21ca63a7d59bdb9026da661";

// RandomnessRequest event selector
const requestSelector =
  "0x00e3e1c077138abb6d570b1a7ba425f5479b12f50a78a72be680167d4cf79c48";

// RandomnessStatusChange event selector
const statusChangeSelector =
  "0x015510b399942790499934b72bc68b883f0905dee5da5aa36e489c9ffb096b8c";

export const config = {
  streamUrl: "https://pragma-mainnet.starknet.a5a.ch",
  startingBlock: Number(Deno.env.get("STARTING_BLOCK") || 534408),
  network: "starknet",
  batchSize: 1,
  finality: "DATA_STATUS_PENDING",
  filter: {
    header: { weak: true },
    events: [
      {
        fromAddress,
        keys: [requestSelector],
        includeTransaction: true,
        includeReceipt: false,
      },
      {
        fromAddress,
        keys: [statusChangeSelector],
        includeTransaction: true,
        includeReceipt: false,
      },
    ],
  },
  sinkType: "postgres",
  sinkOptions: {
    entityMode: true,
  },
};

export default function transform({ header, events }) {
  const { timestamp } = header;
  return events.flatMap(({ event, transaction }) => {
    if (event.keys[0] == requestSelector) {
      // Initialize request entity.
      const [
        requestId,
        callerAddress,
        seed,
        minimumBlockNumber,
        callbackAddress,
        callbackFeeLimit,
        numWords,
      ] = event.data;
      return {
        insert: {
          data_id: `${transaction.meta.hash}_${event.index}`,
          network: "starknet-mainnet",
          request_id: +requestId,
          seed: +seed,
          created_at: timestamp,
          created_at_tx: transaction.meta.hash,
          minimum_block_number: +minimumBlockNumber,
          callback_address: callbackAddress,
          callback_fee_limit: +callbackFeeLimit,
          num_words: +numWords,
          requestor_address: callerAddress,
          updated_at: timestamp,
          updated_at_tx: transaction.meta.hash,
          status: 0,
        },
      };
    } else if (event.keys[0] == statusChangeSelector) {
      // Update request entity.
      const callerAddress = event.data[0];
      const requestId = event.data[1];
      const status = event.data[2];
      return {
        entity: {
          request_id: +requestId,
          requestor_address: callerAddress,
        },
        update: {
          status: +status,
          updated_at: timestamp,
          updated_at_tx: transaction.meta.hash,
        },
      };
    } else {
      // Do nothing.
      return [];
    }
  });
}
