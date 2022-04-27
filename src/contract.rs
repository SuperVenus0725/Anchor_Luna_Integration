use cosmwasm_std::{
    entry_point, to_binary,  Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,CosmosMsg,WasmMsg,Decimal,QueryRequest,WasmQuery,Decimal256,BankMsg
};

use cw2::set_contract_version;
use cw20::{Cw20QueryMsg,BalanceResponse, Cw20ExecuteMsg};

use crate::error::{ContractError};
use crate::msg::{ ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State,CONFIG};
use crate::anchor::{ExecuteMsg as AnchorExecuteMsg, EpochStateResponse, QueryMsg as AnchorQueryMsg};

use terra_cosmwasm::{create_swap_msg,TerraMsgWrapper};

const CONTRACT_NAME: &str = "Anchor_Luna";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    if msg.anchor_portion+msg.luna_portion != Decimal::one() {
        return Err(ContractError::PortionError { })
    }
    let state = State {
       total_deposit:Uint128::new(0),
       anchor_portion :msg.anchor_portion,
       luna_portion :msg.luna_portion,
       anchor_address : msg.anchor_address,
       token_address : msg.token_address,
       denom : msg.denom,
       owner : _info.sender.to_string()
    };
    CONFIG.save(deps.storage,&state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TerraMsgWrapper>, ContractError> {
    
    match msg {
    ExecuteMsg::Deposit {} => execute_deposit(deps,env,info),
    ExecuteMsg::Withdraw {amount} => execute_withdraw(deps,env,info,amount),
    ExecuteMsg::SendToWallet { amount }=> execute_send_to_wallet(deps,env,info,amount),
    ExecuteMsg::SetOwner {address} => execute_set_owner(deps,env,info,address),
    ExecuteMsg::ChangePortion { anchor_portion,luna_portion } => execute_change_portion(deps,env,info,anchor_portion,luna_portion)
    }
}

pub fn execute_deposit(
    deps:DepsMut,
    _env:Env,
    info:MessageInfo
)->Result<Response<TerraMsgWrapper>, ContractError> {
    let mut state = CONFIG.load(deps.storage)?;
    
    let deposit_amount= info
        .funds
        .iter()
        .find(|c| c.denom == state.denom)
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);

    let anchor_deposit = deposit_amount*state.anchor_portion;
    let luna_swap = deposit_amount*state.luna_portion;
   
    let  total_deposit = state.total_deposit+ anchor_deposit;
    state.total_deposit = total_deposit;

    CONFIG.save(deps.storage,&state)?;

    let msg = create_swap_msg(Coin{
        denom:"uusd".to_string(),
        amount : luna_swap
    }, "uluna".to_string());

    
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.anchor_address,
            funds: vec![Coin{
                denom:state.denom,
                amount:anchor_deposit
            }],
            msg: to_binary(&AnchorExecuteMsg::DepositStable  {
            })?,
        }))
        .add_message(msg)
    )
}

pub fn execute_withdraw(
    _deps:DepsMut,
    _env:Env,
    _info:MessageInfo,
    amount:Uint128
)->Result<Response<TerraMsgWrapper>, ContractError> {
    let state = CONFIG.load(_deps.storage)?;
   
    Ok(Response::new()
     .add_message(
         CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: state.token_address,
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                 contract: state.anchor_address, 
                 amount: amount, 
                 msg: to_binary(&Cw20QueryMsg::Balance { address: _env.contract.address.to_string() })? })?,
        })
     ))
}


pub fn execute_send_to_wallet(
    _deps:DepsMut,
    _env:Env,
    _info:MessageInfo,
    amount:Uint128
)->Result<Response<TerraMsgWrapper>, ContractError> {
    let state = CONFIG.load(_deps.storage)?;
    if _info.sender.to_string() != state.owner{
        return Err(ContractError::Unauthorized { })
    }
    Ok(Response::new()
     .add_message(
           CosmosMsg::Bank(BankMsg::Send {
            to_address: _info.sender.to_string(),
            amount: vec![
                Coin {
                    denom: state.denom.clone(),
                    amount: amount,
                },
            ],
        }),
     ))
}

fn execute_set_owner(
    deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    address: String,
) -> Result<Response<TerraMsgWrapper>, ContractError> {
    let mut state = CONFIG.load(deps.storage)?;

    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    deps.api.addr_validate(&address)?;
    state.owner = address;
    CONFIG.save(deps.storage,&state)?;
    Ok(Response::default())
}


fn execute_change_portion(
    deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    anchor_portion: Decimal,
    luna_portion : Decimal
) -> Result<Response<TerraMsgWrapper>, ContractError> {
    let mut state = CONFIG.load(deps.storage)?;
    if info.sender.to_string()!=state.owner{
        return Err(ContractError::Unauthorized { })
    } 

    if anchor_portion + luna_portion != Decimal::one() {
        return Err(ContractError::PortionError {  });
    }
    state.anchor_portion = anchor_portion;
    state.luna_portion = luna_portion;
    CONFIG.save(deps.storage,&state)?;
    Ok(Response::default())
}



#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo {} => to_binary(&query_state_info(deps)?),
        QueryMsg::GetEpochState {} => to_binary(&query_epoch_state(deps)?),
        QueryMsg::GetAustBalance {} => to_binary(&query_aust_balance(deps,_env)?),
    }
}

pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state =  CONFIG.load(deps.storage)?;
    Ok(state)
}

pub fn query_epoch_state(deps:Deps) -> StdResult<EpochStateResponse>{
    let state = CONFIG.load(deps.storage)?;
    let epoch_state =   deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.anchor_address,
        msg: to_binary(&AnchorQueryMsg::EpochState {block_height : None ,distributed_interest :None })?,
    }))?;
    Ok(epoch_state)
}


pub fn query_aust_balance(deps:Deps,env: Env) -> StdResult<BalanceResponse>{
    let state = CONFIG.load(deps.storage)?;
    let balance =   deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.token_address,
        msg: to_binary(&Cw20QueryMsg::Balance {address: env.contract.address.to_string()})?,
    }))?;
    Ok(balance)
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{ CosmosMsg, Coin, Uint256,Decimal256};

    #[test]
    fn deposit() {
        let mut deps = mock_dependencies(&[]);
        let instantiate_msg = InstantiateMsg {
            anchor_portion:Decimal::from_ratio(8 as u128, 10 as u128),
            luna_portion:Decimal::from_ratio(2 as u128,10 as u128),
            anchor_address:"anchor_address".to_string(),
            denom : "uusd".to_string(),
            token_address : "token_address".to_string()
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
        let state_info =  query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state_info,State{
            anchor_portion:Decimal::from_ratio(8 as u128, 10 as u128),
            luna_portion : Decimal::from_ratio(2 as u128, 10 as u128),
            anchor_address : "anchor_address".to_string(),
            token_address :"token_address".to_string(),
            total_deposit : Uint128::new(0),
            denom : "uusd".to_string(),
            owner : "creator".to_string()
        });

        let info = mock_info("creator", &[]);
        let message = ExecuteMsg::ChangePortion {
            anchor_portion : Decimal::from_ratio(7 as u128, 10 as u128),
            luna_portion : Decimal::from_ratio(3 as u128, 10 as u128),
        };
        execute(deps.as_mut(), mock_env(), info, message).unwrap();
        
        let state_info =  query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state_info,State{
            anchor_portion:Decimal::from_ratio(7 as u128, 10 as u128),
            luna_portion : Decimal::from_ratio(3 as u128, 10 as u128),
            anchor_address : "anchor_address".to_string(),
            token_address : "token_address".to_string(),
            total_deposit : Uint128::new(0),
            denom : "uusd".to_string(),
            owner : "creator".to_string()
        });

        let x = Uint128::new(50);
        let y = x*Decimal::from_ratio(7 as u128, 10 as u128);
        assert_eq!(Uint128::new(35),y);

        let info = mock_info("creator", &[Coin{
              denom:"uusd".to_string(),
              amount:Uint128::new(50)
        }]);
        let message = ExecuteMsg::SetOwner { address: "creator1".to_string() } ;
        execute(deps.as_mut(), mock_env(), info, message).unwrap();
        let state_info  = query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state_info.owner,"creator1".to_string());

        let info = mock_info("creator", &[Coin{
              denom:"uusd".to_string(),
              amount:Uint128::new(50)
        }]);
        let message = ExecuteMsg::Deposit { };
        let res = execute(deps.as_mut(), mock_env(), info, message).unwrap();
        let state_info =  query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state_info.total_deposit , Uint128::new(35));
        assert_eq!(res.messages.len(),2);
        assert_eq!(res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "anchor_address".to_string(),
                funds: vec![Coin{
                    denom:"uusd".to_string(),
                    amount:Uint128::new(35)
                }],
                msg: to_binary(&AnchorExecuteMsg::DepositStable {
            }).unwrap(),
        }));
        let info = mock_info("creator", &[]);
        let message = ExecuteMsg::Withdraw { amount: Uint128::new(100) }  ;
        let res = execute(deps.as_mut(), mock_env(), info, message).unwrap();
        assert_eq!(1,res.messages.len());
        assert_eq!(res.messages[0].msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "token_address".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Send {
                 contract: "anchor_address".to_string(), 
                 amount: Uint128::new(100), 
                 msg: to_binary(&Cw20QueryMsg::Balance { address: mock_env().contract.address.to_string() }).unwrap() }).unwrap(),
        }));

        let info = mock_info("creator1", &[]);
        let message = ExecuteMsg::SendToWallet { amount: Uint128::new(100) }  ;
        let res = execute(deps.as_mut(), mock_env(), info, message).unwrap();
        assert_eq!(1,res.messages.len());
        assert_eq!(res.messages[0].msg,
          CosmosMsg::Bank(BankMsg::Send {
            to_address: "creator1".to_string(),
            amount: vec![
                Coin {
                    denom: "uusd".to_string(),
                    amount: Uint128::new(100),
                },
            ],
        }),);
    }
}
