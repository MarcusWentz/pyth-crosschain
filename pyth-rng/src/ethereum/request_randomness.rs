use std::error::Error;
use std::sync::Arc;

use ethers::core::types::Address;
use sha3::Digest;

use crate::config::RequestRandomnessOptions;

use super::instantiate_contract_from_opts;

pub async fn request_randomness(opts: &RequestRandomnessOptions) -> Result<(), Box<dyn Error>> {
    let contract = Arc::new(instantiate_contract_from_opts(&opts.ethereum).await?);

    let user_randomness = rand::random::<[u8; 32]>();
    let provider = opts.provider.parse::<Address>()?;

    let sequence_number = contract.request_wrapper(&provider, &user_randomness, false).await?;

    println!("sequence number: {:#?}", sequence_number);

    /*
    if let Some(r) = contract
        .request(provider, hashed_randomness, false)
        .value(200)
        .send()
        .await?
        .await?
    {
        // Extract Log from TransactionReceipt.
        let l: RawLog = r.logs[0].clone().into();
        if let PythRandomEvents::RequestedFilter(r) = super::PythRandomEvents::decode_log(&l)? {
            let sequence_number = r.request.sequence_number;
            if let Some(r) = contract
                .reveal(
                    provider,
                    sequence_number,
                    user_randomness,
                    chain.reveal_ith(sequence_number as usize)?,
                )
                .send()
                .await?
                .await?
            {
                if let PythRandomEvents::RevealedFilter(r) =
                    super::PythRandomEvents::decode_log(&r.logs[0].clone().into())?
                {
                    println!("Random number: {:#?}", r);
                }
            }
        }
    }
     */

    Ok(())
}
