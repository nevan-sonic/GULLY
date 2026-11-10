"""
GULLY Rush & Token Supply Simulator (60-Day Modeling)
Required deliverable for NEAR Hackathon 2026 - Track 11: Memecoins With Mechanics

This script simulates 60 days of community behavior and GULLY Coin (NEP-141) token dynamics
under three real-world scenarios:
  Scenario A: Steady / Low Adoption (Early neighborhood pilot)
  Scenario B: High Adoption & Viral Gully Rush (Active neighborhood milestones)
  Scenario C: Sybil / Farmer Attack Attempt (Collusion attack bounded by protocol rules)
"""

import math
import random

def simulate_scenario(scenario_name, daily_shoppers_range, num_merchants, rush_trigger_chance, redemption_rate, sybil_active=False):
    print("=" * 70)
    print(f"RUNNING SCENARIO: {scenario_name}")
    print("=" * 70)

    # Immutable Economic Parameters
    BASE_MINT_PER_PURCHASE = 10      # 10 GULLY coins per eligible purchase (₹200+)
    MAX_MINTS_PER_USER_DAILY = 2     # Strict Sybil rate limit per merchant
    RUSH_THRESHOLD = 100             # Purchases needed in 48h to trigger Rush
    RUSH_BONUS_PER_USER = 5          # Milestone bonus minted upon Rush activation
    MAX_SUPPLY_CAP = 1_000_000       # Protocol hard cap

    total_supply = 0
    total_minted = 0
    total_burned = 0
    rush_events_triggered = 0

    history = []
    recent_48h_purchases = []

    # Deterministic seed for reproducible testing
    random.seed(42)

    for day in range(1, 61):
        num_shoppers = random.randint(*daily_shoppers_range)
        day_purchases = 0
        day_minted = 0
        day_burned = 0

        # Normal organic shopping
        for shopper_id in range(num_shoppers):
            # 1 to 2 purchases per day
            purchases = random.randint(1, 2)
            day_purchases += purchases
            day_minted += purchases * BASE_MINT_PER_PURCHASE

        # Sybil attack attempt
        if sybil_active:
            # Attacker creates 50 fake purchase attempts
            sybil_attempts = 50
            # Protocol strictly clamps attacker to MAX_MINTS_PER_USER_DAILY per merchant
            sybil_accepted = min(sybil_attempts, MAX_MINTS_PER_USER_DAILY * num_merchants)
            day_purchases += sybil_accepted
            day_minted += sybil_accepted * BASE_MINT_PER_PURCHASE

        # Track 48-hour rolling window for Gully Rush
        recent_48h_purchases.append(day_purchases)
        if len(recent_48h_purchases) > 2:
            recent_48h_purchases.pop(0)

        rolling_purchases = sum(recent_48h_purchases)

        # Check Rush Milestone
        rush_activated_today = False
        if rolling_purchases >= RUSH_THRESHOLD and random.random() < rush_trigger_chance:
            rush_events_triggered += 1
            rush_activated_today = True
            # Bonus minted to active shoppers
            bonus_mint = num_shoppers * RUSH_BONUS_PER_USER
            day_minted += bonus_mint
            # Reset rolling window upon milestone trigger
            recent_48h_purchases = []

        # Redemptions (burning coins for discounts at merchants)
        available_coins = total_supply + day_minted
        if available_coins > 0:
            target_burn = int(available_coins * redemption_rate)
            # If rush is active, redemption velocity spikes by 50%
            if rush_activated_today:
                target_burn = int(target_burn * 1.5)
            day_burned = min(available_coins, target_burn)

        # Supply update: Supply = Old + Minted - Burned
        total_supply = total_supply + day_minted - day_burned
        total_minted += day_minted
        total_burned += day_burned

        history.append({
            "day": day,
            "shoppers": num_shoppers,
            "purchases": day_purchases,
            "minted": day_minted,
            "burned": day_burned,
            "supply": total_supply,
            "rush_active": rush_activated_today
        })

    print(f"60-Day Summary for {scenario_name}:")
    print(f"  Final Circulating Supply: {total_supply:,.0f} GULLY")
    print(f"  Total Coins Minted:       {total_minted:,.0f} GULLY")
    print(f"  Total Coins Burned:       {total_burned:,.0f} GULLY (Burn Ratio: {total_burned/total_minted*100:.1f}%)")
    print(f"  Neighborhood Rush Events: {rush_events_triggered} times triggered")
    print("-" * 70)
    print(f"{'Day':<5} | {'Shoppers':<10} | {'Minted':<10} | {'Burned':<10} | {'Net Supply':<12} | {'Rush Active'}")
    print("-" * 70)
    for row in history[::5]:  # Sample every 5 days
        print(f"{row['day']:<5} | {row['shoppers']:<10} | {row['minted']:<10} | {row['burned']:<10} | {row['supply']:<12} | {'YES (RUSH)' if row['rush_active'] else 'No'}")
    print("=" * 70 + "\n")
    return history

if __name__ == "__main__":
    print("\n--- GULLY COIN (NEP-141) 60-DAY ECONOMIC SIMULATION ---")
    
    # 1. Steady Adoption
    sim_a = simulate_scenario(
        scenario_name="Scenario A: Steady Low Adoption (Pilot)",
        daily_shoppers_range=(15, 30),
        num_merchants=3,
        rush_trigger_chance=0.4,
        redemption_rate=0.08,
        sybil_active=False
    )

    # 2. High Adoption / Rush Driven
    sim_b = simulate_scenario(
        scenario_name="Scenario B: High Adoption (Active Gully Rush Events)",
        daily_shoppers_range=(60, 120),
        num_merchants=8,
        rush_trigger_chance=0.85,
        redemption_rate=0.12,
        sybil_active=False
    )

    # 3. Sybil Attack Attempt
    sim_c = simulate_scenario(
        scenario_name="Scenario C: Sybil Collusion Attack Attempt",
        daily_shoppers_range=(20, 40),
        num_merchants=3,
        rush_trigger_chance=0.5,
        redemption_rate=0.08,
        sybil_active=True
    )
