import { Context, RNG, PersistentUnorderedMap } from "near-sdk-as";

import { AccountId } from "../../utils";

type Code = string;

const ONE_SECOND: u64 = 1_000_000_000

/**
 * Proof of presence contract
 * 
 * @param {AccountId} owner
 * @param {i16} [visitor_cooldown=60] - cooldown between visits in seconds
 * @param {bool} [track_visitors=true] - should this contract track all visitors
 * @throws if cooldown is not in range between 1 second and 5 minutes
 */
@nearBindgen
export class Contract {

  private owner: AccountId;
  private valid_code: Code;
  private should_track_visitors: bool;
  private visitors: PersistentUnorderedMap<AccountId, u16> = new PersistentUnorderedMap<AccountId, u16>("v");
  private visitor_page_size: u8 = 10;
  private last_visitor: AccountId;
  private last_visit: u64;
  private isActive: bool = true;


  // TODO: cooldown this doesn't track multiple visit timers so any new visitor will reset timer
  // FIX: decide if we want to track last visited time for each visitor and measure against that
  private cooldown: u64;

  constructor(
    owner: AccountId,
    visitor_cooldown: i16 = 60, // seconds
    track_visitors: bool = true
  ) {
    // track owner to guard owner methods
    this.owner = owner;

    // capture configuration for cooldown between repeat visits by a single account
    this.assert_reasonable_cooldown(visitor_cooldown);
    this.cooldown = visitor_cooldown * ONE_SECOND;

    // turn visitor tracking on and off
    this.should_track_visitors = track_visitors;

    // start with the first valid code
    this.valid_code = this.generate_code();
  }

  // --------------------------------------------------------------------------
  // Public VIEW methods
  // --------------------------------------------------------------------------

  /**
   * An inactive contract does not handle code confirmations
   *
   * @returns {bool} state of the contract as active / inactive
   */
  get_active(): bool {
    return this.isActive;
  }

  /**
   * @returns {AccountId} of contract owner
   */
  get_owner(): AccountId {
    return this.owner;
  }

  /**
   * There can only be one valid code at a time
   * 
   * @returns {string} currently valid code
   */
  get_code(): string {
    return this.valid_code;
  }

  // --------------------------------------------------------------------------
  // Owner VIEW methods
  // --------------------------------------------------------------------------

  /** 
   * @returns {AccountId} of last visitor
   * @throws if called by non-owner
   */
  get_last_visitor(): AccountId {
    this.assert_owner()
    return this.last_visitor;
  }

  // --------------------------------------------------------------------------
  // Public CHANGE methods
  // --------------------------------------------------------------------------

  /**
   * Confirm PoP code to establish proof of presence
   *
   * @param {Code} code - proof of presence code to verify
   * @returns {bool} true or fails
   * @throws if contract is inactive
   * @throws if code is invalid
   * @throws if code is no longer active
   * @throws if visitor is the same as previous (e.g. visits twice in a row)
   * @throws if cooldown time after previous visit hasn't passed
   */
  @mutateState()
  confirm_code(code: Code): true {
    assert(this.isActive, "The system is currently inactive")

    this.assert_valid_format(code)
    assert(this.valid_code == code, "This code is no longer active")

    if (this.should_track_visitors) {
      // capture record of visitor
      this.update_visitor_record()
    }

    // update the valid code to a new one
    this.valid_code = this.generate_code()
    return true;
  }

  // --------------------------------------------------------------------------
  // Owner CHANGE methods
  // --------------------------------------------------------------------------

  /**
   * This method will fail if the contract is not configured to track
   * visitors. Also, if the owner of the contract has cleared visitor
   * data since this guest confirmed a code then any record of their visit is lost
   *
   * @param {AccountId} guest - account that may have visited in the past
   * @returns {bool} indicating whether the guest has visited
   * @throws if called by non-owner
   * @throws if contract is not tracking visitors
   */
  get_has_visited(guest: AccountId): bool {
    this.assert_owner()
    assert(this.should_track_visitors, "This PoPskl is not configured to track visitors")
    return this.visitors.contains(guest);
  }

  /**
   * This method may not return any data if the owner of the contract
   * has recently cleared out visitor data
   *
   * @returns {Array<AccountId>} list of visitors
   * @throws if called by non-owner
   */
  // TODO: limit using pagination
  get_visitors(page: u16 = 1): Array<AccountId> {
    this.assert_owner()
    const start = max(0, this.visitors.length - (page * this.visitor_page_size))
    return this.visitors.keys(start, this.visitors.length)
  }

  /**
   * This method may not return accurate data if the owner of the contract
   * has cleared out visitor data in the lifetime of this contract

   * @param {AccountId} guest - account for which we want visit count
   * @returns {u16} representing the number of visits tracked for this guest
   * @throws if called by non-owner
   */
  get_visit_count(guest: AccountId): u16 {
    this.assert_owner()
    if (this.visitors.contains(guest)) {
      return this.visitors.getSome(guest)
    } else {
      return 0
    }
  }

  /**
   * Clear all visitor data. Useful as contract storage staking costs grow large
   *
   * @returns true always
   * @throws if called by non-owner
   */
  @mutateState()
  clear_visitor_records(): true {
    this.assert_owner()
    this.visitors.clear()
    this.last_visitor = '';
    return true
  }

  /**
   * Toggle the active state of the contract. An inactive contract
   * does not handle code confirmations
   *
   * @returns {bool} state of contract as active / inactive
   * @throws if called by non-owner
   */
  @mutateState()
  toggle_active(): bool {
    this.assert_owner()
    this.isActive = !this.isActive
    return this.isActive
  }

  /**
   *
   * @param cooldown number of seconds between repeat visits
   * @returns state of contract as active / inactive
   * @throws if called by non-owner
   * @throws if cooldown is not in range between 1 second and 5 minutes
   */
  @mutateState()
  set_cooldown(cooldown: i16): bool {
    this.assert_owner()
    this.assert_reasonable_cooldown(cooldown)
    this.cooldown = cooldown * ONE_SECOND
    return this.isActive
  }

  // --------------------------------------------------------------------------
  // Private methods
  // --------------------------------------------------------------------------

  private update_visitor_record(): void {
    const sender = Context.sender

    this.assert_not_immediate_duplicate()

    if (this.visitors.contains(sender)) {
      const visits = this.visitors.getSome(sender)
      this.visitors.set(sender, visits + 1);
    } else {
      this.visitors.set(sender, 1);
    }

    this.last_visitor = sender
    this.last_visit = Context.blockTimestamp;
  }

  private generate_code(): string {
    const rng = new RNG<u32>(1, u32.MAX_VALUE)

    let data = Context.contractName;
    data += '|' + Context.blockIndex.toString();
    data += '|' + Context.blockTimestamp.toString();
    data += '|' + rng.next().toString();

    return data
  }

  private assert_valid_format(code: Code): bool {
    const parts = code.split('|')
    // must have four parts
    assert(parts.length == 4, "Invalid Code")
    assert(parts[0] == Context.contractName, "Invalid Code")
    // second part have the value of a previous block index
    assert(<u64>parseInt(parts[1]) < Context.blockIndex, "Invalid Code")
    // fourth part must have value less than max random u32
    assert(parseInt(parts[3]) < u32.MAX_VALUE, "Invalid Code")
    return true
  }

  private assert_owner(): void {
    const caller = Context.predecessor
    assert(this.owner == caller, "Only the owner of this contract may call this method");
  }

  private assert_not_immediate_duplicate(): void {
    const newSender = Context.sender != this.last_visitor
    const within_one_minute = this.last_visit + this.cooldown > Context.blockTimestamp
    assert(newSender || !within_one_minute, "Duplicate proof of presence detected")
  }

  private assert_reasonable_cooldown(cooldown: i32): void {
    assert(cooldown > 0 && cooldown <= 5 * 60, "Cooldown must be 5 minutes or less (measured in seconds)")
  }
}
