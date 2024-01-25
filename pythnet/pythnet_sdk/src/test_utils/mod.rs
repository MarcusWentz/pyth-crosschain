use {
    crate::{
        accumulators::{
            merkle::MerkleTree,
            Accumulator,
        },
        hashers::keccak256_160::Keccak160,
        messages::{
            Message,
            PriceFeedMessage,
        },
        wire::{
            to_vec,
            v1::{
                AccumulatorUpdateData,
                MerklePriceUpdate,
                Proof,
                WormholeMerkleRoot,
                WormholeMessage,
                WormholePayload,
            },
            PrefixedVec,
        },
    },
    byteorder::BigEndian,
    serde_wormhole::RawMessage,
    std::io::{
        Cursor,
        Write,
    },
    wormhole_sdk::{
        Address,
        Chain,
        Vaa,
    },
};

pub const EMITTER_CHAIN: u16 = 0;
pub const DEFAULT_SEQUENCE: u64 = 2;

pub fn default_emitter_addr() -> [u8; 32] {
    [1u8; 32]
}

pub fn default_governance_addr() -> [u8; 32] {
    [2u8; 32]
}

pub fn default_new_governance_addr() -> [u8; 32] {
    [3u8; 32]
}

pub fn default_wrong_emitter_addr() -> [u8; 32] {
    [4u8; 32]
}

pub fn create_dummy_price_feed_message(value: i64) -> Message {
    let mut dummy_id = [0; 32];
    dummy_id[0] = value as u8;
    let msg = PriceFeedMessage {
        feed_id:           dummy_id,
        price:             value,
        conf:              value as u64,
        exponent:          value as i32,
        publish_time:      value,
        prev_publish_time: value,
        ema_price:         value,
        ema_conf:          value as u64,
    };
    Message::PriceFeedMessage(msg)
}

pub fn create_accumulator_message(
    all_feeds: &[Message],
    updates: &[Message],
    corrupt_wormhole_message: bool,
) -> Vec<u8> {
    let all_feeds_bytes: Vec<_> = all_feeds
        .iter()
        .map(|f| to_vec::<_, BigEndian>(f).unwrap())
        .collect();
    let all_feeds_bytes_refs: Vec<_> = all_feeds_bytes.iter().map(|f| f.as_ref()).collect();
    let tree = MerkleTree::<Keccak160>::new(all_feeds_bytes_refs.as_slice()).unwrap();
    let mut price_updates: Vec<MerklePriceUpdate> = vec![];
    for update in updates {
        let proof = tree
            .prove(&to_vec::<_, BigEndian>(update).unwrap())
            .unwrap();
        price_updates.push(MerklePriceUpdate {
            message: PrefixedVec::from(to_vec::<_, BigEndian>(update).unwrap()),
            proof,
        });
    }
    create_accumulator_message_from_updates(
        price_updates,
        tree,
        corrupt_wormhole_message,
        default_emitter_addr(),
        EMITTER_CHAIN,
    )
}

pub fn create_accumulator_message_from_updates(
    price_updates: Vec<MerklePriceUpdate>,
    tree: MerkleTree<Keccak160>,
    corrupt_wormhole_message: bool,
    emitter_address: [u8; 32],
    emitter_chain: u16,
) -> Vec<u8> {
    let mut root_hash = [0u8; 20];
    root_hash.copy_from_slice(&to_vec::<_, BigEndian>(&tree.root).unwrap()[..20]);
    let wormhole_message = WormholeMessage::new(WormholePayload::Merkle(WormholeMerkleRoot {
        slot:      0,
        ring_size: 0,
        root:      root_hash,
    }));

    let mut vaa_payload = to_vec::<_, BigEndian>(&wormhole_message).unwrap();
    if corrupt_wormhole_message {
        vaa_payload[0] = 0;
    }

    let vaa = create_vaa_from_payload(
        &vaa_payload,
        emitter_address,
        emitter_chain,
        DEFAULT_SEQUENCE,
    );

    let accumulator_update_data = AccumulatorUpdateData::new(Proof::WormholeMerkle {
        vaa:     PrefixedVec::from(serde_wormhole::to_vec(&vaa).unwrap()),
        updates: price_updates,
    });

    to_vec::<_, BigEndian>(&accumulator_update_data).unwrap()
}

pub fn create_vaa_from_payload(
    payload: &[u8],
    emitter_address: [u8; 32],
    emitter_chain: u16,
    sequence: u64,
) -> Vaa<Box<RawMessage>> {
    let vaa: Vaa<Box<RawMessage>> = Vaa {
        emitter_chain: Chain::from(emitter_chain),
        emitter_address: Address(emitter_address),
        sequence,
        payload: <Box<RawMessage>>::from(payload.to_vec()),
        ..Default::default()
    };
    vaa
}