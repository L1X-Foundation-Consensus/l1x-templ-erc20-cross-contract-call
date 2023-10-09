// SPDX-License-Identifier: MIT
pragma solidity ^0.7.0;
import "@balancer-labs/v2-interfaces/contracts/solidity-utils/openzeppelin/IERC20.sol";

contract MYERC20 is IERC20 {
    string public name;
    string public symbol;
    uint8 public decimals;

    mapping(address => uint256) balances;
    mapping(address => mapping(address => uint256)) allowed;
    uint256 totalSupply_;
    address public owner;

    modifier onlyOwner() {
        require(msg.sender == owner, "Not contract from");
        _;
    }

    constructor(
        uint256 initialSupply,
        string memory _name,
        string memory _symbol,
        uint8 _decimals
    ) {
        name = _name;
        symbol = _symbol;
        decimals = _decimals;
        totalSupply_ = initialSupply * (10**uint256(decimals));
        balances[msg.sender] = totalSupply_;
        owner = msg.sender;
        emit Transfer(address(0), msg.sender, totalSupply_);
    }

    function totalSupply() public view override returns (uint256) {
        return totalSupply_;
    }

    function changeOwner(address newOwner) public onlyOwner {
        owner = newOwner;
    }

    function mint(address account, uint256 amount) public onlyOwner {
        require(account != address(0), "ERC20: mint to the zero address");
        totalSupply_ = totalSupply_ + amount;
        balances[account] = balances[account] + amount;
        emit Transfer(address(0), account, amount);
    }

    function burn(address account, uint256 amount) public onlyOwner {
        require(account != address(0), "ERC20: burn from the zero address");
        require(balances[account] >= amount, "ERC20: burn amount exceeds balance");
        totalSupply_ = totalSupply_ - amount;
        balances[account] = balances[account] - amount;
        emit Transfer(account, address(0), amount);
    }

    function balanceOf(address tokenOwner) public view override returns (uint256) {
        return balances[tokenOwner];
    }

    function transfer(address receiver, uint256 numTokens) public override returns (bool) {
        require(numTokens <= balances[msg.sender]);
        balances[msg.sender] = balances[msg.sender] - numTokens;
        balances[receiver] = balances[receiver] + numTokens;
        emit Transfer(msg.sender, receiver, numTokens);
        return true;
    }

    function approve(address delegate, uint256 numTokens) public override returns (bool) {
        allowed[msg.sender][delegate] = numTokens;
        emit Approval(msg.sender, delegate, numTokens);
        return true;
    }

    function allowance(address from, address delegate) public view override returns (uint256) {
        return allowed[from][delegate];
    }

    function transferFrom(
        address from,
        address buyer,
        uint256 numTokens
    ) public override returns (bool) {
        require(numTokens <= balances[from]);
        require(numTokens <= allowed[from][msg.sender]);

        balances[from] = balances[from] - numTokens;
        allowed[from][msg.sender] = allowed[from][msg.sender] - numTokens;
        balances[buyer] = balances[buyer] + numTokens;
        emit Transfer(from, buyer, numTokens);
        return true;
    }
}
