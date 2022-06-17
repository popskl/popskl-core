import { VMContext } from "near-sdk-as";

import { Contract } from "../assembly/index";
import { ONE_NEAR } from "../../utils";

const contract = "popskl"
const owner = "alice"
const player1 = "bob"
const player2 = "carol"

let popskl: Contract

beforeEach(() => {
  VMContext.setCurrent_account_id(contract)
  VMContext.setAccount_balance(ONE_NEAR) // resolves HostError(BalanceExceeded)
  popskl = new Contract(owner)
})

// --------------------------------------------
// --------------------------------------------
// VIEW method tests
// --------------------------------------------
// --------------------------------------------

// --------------------------------------------
// Contract Metadata
// --------------------------------------------
describe("Contract", () => {

  it("can be initialized with owner", () => {
    // who owns this lottery? -> AccountId
    expect(popskl.get_owner()).toBe(owner)
  });

  it("is active when initialized", () => {
    // is the lottery still active? -> bool
    expect(popskl.get_active()).toBe(true)
  })
})
