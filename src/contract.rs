use crate::state::{load_from_store, save_to_store};

use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdError, StdResult, Storage,
};

use crate::msg::{HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg};
use crate::state::{config, config_read, File, Metadata, MetadataSchema, MetadataStorage, State};

use libipld::block::Block;
use libipld::ipld;
use libipld::ipld::Ipld;
use libipld::cid::multihash::Code;
use libipld::store::DefaultParams;
use libipld::Cid;
use libipld::cbor::DagCborCodec;

use std::str::FromStr;

type IpldBlock = libipld::block::Block<DefaultParams>;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        paused: false,
        owner: deps.api.canonical_address(&env.message.sender)?,
    };

    config(&mut deps.storage).save(&state)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleAnswer> {
    match msg {
        HandleMsg::AddFile {
            path,
            content_type,
            time,
            content,
            mode,
        } => add_file(deps, env, path, content_type, time, content, mode),
        HandleMsg::AddMetadata { data } => add_metadata(deps, env, data),
    }
}

pub fn add_metadata<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    data: MetadataSchema,
) -> StdResult<HandleAnswer> {
    let links: Vec<_> = data
        .links
        .iter()
        .map(|l| Ipld::Link(Cid::from_str(l).unwrap()))
        .collect();
    //links.iter().map(|l| println!("{}", l.to_string()));
    // let links2 = data.links;

    // for x in links2.iter() {
    //     links2[x] = links2[x];
    // };

    let block = Block::<DefaultParams>::encode(
        DagCborCodec,
        Code::Sha2_256,
        &ipld!({
            "name":data.name,
            "description": data.description,
            "image": data.image,
            "links": links,
        }),
    )
    .unwrap();

    let cid = block.cid().to_string();
    let data = block.data().to_vec();

    let callback = HandleAnswer::AddMetadata { cid: cid.clone() };
    let id = cid.into_bytes();
    save_to_store(&mut deps.storage, &id, &data)?;
    Ok(callback)
}

pub fn add_file<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    path: String,
    content_type: String,
    time: u64,
    content: Vec<u8>,
    mode: String,
) -> StdResult<HandleAnswer> {
    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;

    let block = Block::<DefaultParams>::encode(
        DagCborCodec,
        Code::Sha2_256,
        &ipld!({
            "owner": sender_address_raw.to_string(),
            "path": path,
            "type": content_type,
            "content": content,
            "time": time,
        }),
    )
    .unwrap();

    let cid = block.cid().to_string();
    let data = block.data().to_vec();

    let callback = HandleAnswer::AddFile { cid: cid.clone() };
    let id = cid.into_bytes();
    save_to_store(&mut deps.storage, &id, &data)?;
    Ok(callback)
}

fn get_metadata<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    cid: String,
) -> StdResult<QueryAnswer> {
    let try_cid = Cid::new_v0(
        libipld::cid::multihash::MultihashGeneric::from_bytes(&cid.into_bytes()).unwrap(),
    )
    .unwrap();

    if try_cid.codec() > 0 {
        panic!("AAAaaaaa!!!!");
    }

    let result = load_from_store(&deps.storage, &try_cid.to_bytes());
    // let block = <libipld::Block<libipld::DefaultParams>::encode(
    // try_cid,
    // result.unwrap(),
    // ).unwrap();
    //let block = IpldBlock::new{
    //    cid: try_cid,
    //    data: result.unwrap(),
    //}
    //.unwrap();
    
    let block = IpldBlock::new(try_cid, result.unwrap()).unwrap();
    
    let response = QueryAnswer::GetMetadata {
        data: block.data().to_vec(),
    };

    Ok(response)
}

fn get_files<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    cid: String,
) -> StdResult<QueryAnswer> {
    let try_cid = Cid::new_v0(
        libipld::cid::multihash::MultihashGeneric::from_bytes(&cid.into_bytes()).unwrap(),
    )
    .unwrap();

    if try_cid.codec() > 0 {
        panic!("AAAaaaaa!!!!");
    }

    let result = load_from_store(&deps.storage, &try_cid.to_bytes());
    
    let block = IpldBlock::new(try_cid, result.unwrap()).unwrap();
    //let block = IpldBlock::encode(IpldCodec::Raw, Code::Blake3_256, &result).unwrap();
    
    //let block_decoded =  block.decode::<DagCborCodec, _>().unwrap();
    //block2 = block.data().to_vec();

    let response = QueryAnswer::GetFile {
        data: block.data().to_vec(),
    };

    Ok(response)
}

// pub fn query<S: Storage, A: Api, Q: Querier>(
// deps: &Extern<S, A, Q>,
// msg: QueryMsg,
// ) -> StdResult<HandleAnswer> {
// match msg {
// QueryMsg::GetMetadata { cid } => get_metadata(deps, cid),
// QueryMsg::GetFile { cid } => get_file(deps, msg, cid),
// }
// }

// fn query_count<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<CountResponse> {
// let state = config_read(&deps.storage).load()?;
// Ok(CountResponse { count: state.count })
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env};
//     use cosmwasm_std::{coins, from_binary, StdError};

//     #[test]
//     fn proper_initialization() {
//         let mut deps = mock_dependencies(20, &[]);

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(1000, "earth"));

//         // we can just call .unwrap() to assert this was a success
//         let res = init(&mut deps, env, msg).unwrap();
//         assert_eq!(0, res.messages.len());

//         // it worked, let's query the state
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(17, value.count);
//     }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies(20, &coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(2, "token"));
//         let _res = init(&mut deps, env, msg).unwrap();

//         // anyone can increment
//         let env = mock_env("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Increment {};
//         let _res = handle(&mut deps, env, msg).unwrap();

//         // should increase counter by 1
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies(20, &coins(2, "token"));

//         let msg = InitMsg { count: 17 };
//         let env = mock_env("creator", &coins(2, "token"));
//         let _res = init(&mut deps, env, msg).unwrap();

//         // not anyone can reset
//         let unauth_env = mock_env("anyone", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let res = handle(&mut deps, unauth_env, msg);
//         match res {
//             Err(StdError::Unauthorized { .. }) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_env = mock_env("creator", &coins(2, "token"));
//         let msg = HandleMsg::Reset { count: 5 };
//         let _res = handle(&mut deps, auth_env, msg).unwrap();

//         // should now be 5
//         let res = query(&deps, QueryMsg::GetCount {}).unwrap();
//         let value: CountResponse = from_binary(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
