// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title InheritX
 * @dev Contract for managing inheritance plans with emergency exit functionality
 */
contract InheritX is Ownable {
    using SafeERC20 for IERC20;

    struct Plan {
        address owner;
        IERC20 token;
        uint256 principal;
        uint256 accumulatedYield;
        uint256 livenessExpiration;
        bool isActive;
        uint256 createdAt;
    }

    mapping(uint256 => Plan) public plans;
    uint256 public planCounter;
    uint256 public constant DEFAULT_LIVENESS_PERIOD = 30 days;

    event PlanCreated(
        uint256 indexed planId,
        address indexed owner,
        address token,
        uint256 principal,
        uint256 livenessExpiration
    );

    event YieldAccumulated(uint256 indexed planId, uint256 amount);

    event PlanWithdrawn(
        uint256 indexed planId,
        address indexed owner,
        uint256 principal,
        uint256 yield,
        uint256 totalWithdrawn
    );

    event PlanClosed(uint256 indexed planId);

    /**
     * @dev Constructor to initialize the contract with deployer as owner
     */
    constructor() Ownable(msg.sender) {}

    /**
     * @dev Create a new inheritance plan
     * @param token The ERC20 token address to lock
     * @param principal The amount of tokens to lock as principal
     * @param livenessPeriod The time period before liveness expires (in seconds)
     */
    function createPlan(
        address token,
        uint256 principal,
        uint256 livenessPeriod
    ) external returns (uint256) {
        require(token != address(0), "Invalid token address");
        require(principal > 0, "Principal must be greater than 0");
        require(livenessPeriod > 0, "Liveness period must be greater than 0");

        uint256 planId = ++planCounter;
        uint256 livenessExpiration = block.timestamp + livenessPeriod;

        // Transfer tokens from caller to contract
        IERC20(token).safeTransferFrom(msg.sender, address(this), principal);

        plans[planId] = Plan({
            owner: msg.sender,
            token: IERC20(token),
            principal: principal,
            accumulatedYield: 0,
            livenessExpiration: livenessExpiration,
            isActive: true,
            createdAt: block.timestamp
        });

        emit PlanCreated(planId, msg.sender, token, principal, livenessExpiration);

        return planId;
    }

    /**
     * @dev Accumulate yield to a plan (can be called by anyone)
     * @param planId The plan identifier
     * @param amount The amount of yield to add
     */
    function accumulateYield(uint256 planId, uint256 amount) external {
        Plan storage plan = plans[planId];
        require(plan.isActive, "Plan is not active");
        require(amount > 0, "Yield amount must be greater than 0");

        // Transfer yield tokens to contract
        plan.token.safeTransferFrom(msg.sender, address(this), amount);
        
        plan.accumulatedYield += amount;

        emit YieldAccumulated(planId, amount);
    }

    /**
     * @dev Emergency exit: withdraw plan and return all locked principal plus accumulated yield
     * @param planId The plan identifier
     * 
     * Requirements:
     * - Caller must be the plan owner (owner signature)
     * - Plan must be active
     * - Current time must be before liveness expiration
     * 
     * Effects:
     * - Transfers principal + accumulated yield back to owner
     * - Deactivates the plan
     */
    function withdrawPlan(uint256 planId) external {
        Plan storage plan = plans[planId];

        // Owner signature verification (msg.sender must be plan owner)
        require(msg.sender == plan.owner, "Only plan owner can withdraw");
        
        // Plan must be active
        require(plan.isActive, "Plan is not active");
        
        // Liveness check: can only withdraw before expiration
        require(block.timestamp < plan.livenessExpiration, "Liveness has expired");

        uint256 totalAmount = plan.principal + plan.accumulatedYield;
        require(totalAmount > 0, "No funds to withdraw");

        // Deactivate plan before transfer to prevent reentrancy
        plan.isActive = false;

        // Transfer all tokens back to owner
        plan.token.safeTransfer(msg.sender, totalAmount);

        emit PlanWithdrawn(
            planId,
            msg.sender,
            plan.principal,
            plan.accumulatedYield,
            totalAmount
        );

        emit PlanClosed(planId);

        // Clear plan data (optional, for gas optimization)
        delete plans[planId];
    }

    /**
     * @dev Get plan details
     * @param planId The plan identifier
     */
    function getPlan(uint256 planId) external view returns (
        address owner,
        address token,
        uint256 principal,
        uint256 accumulatedYield,
        uint256 livenessExpiration,
        bool isActive,
        uint256 createdAt
    ) {
        Plan memory plan = plans[planId];
        return (
            plan.owner,
            address(plan.token),
            plan.principal,
            plan.accumulatedYield,
            plan.livenessExpiration,
            plan.isActive,
            plan.createdAt
        );
    }

    /**
     * @dev Check if a plan can be withdrawn (before liveness expiration)
     * @param planId The plan identifier
     */
    function canWithdraw(uint256 planId) external view returns (bool) {
        Plan memory plan = plans[planId];
        return plan.isActive && block.timestamp < plan.livenessExpiration;
    }

    /**
     * @dev Get total value of a plan (principal + yield)
     * @param planId The plan identifier
     */
    function getPlanTotalValue(uint256 planId) external view returns (uint256) {
        Plan memory plan = plans[planId];
        return plan.principal + plan.accumulatedYield;
    }
}
