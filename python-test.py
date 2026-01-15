import carbontime
import time

print("Initializing CarbonTime...")

# Use the class defined in lib.rs
tracker = carbontime.CarbonTracker(interval_ms=100)

with tracker:
    # Simulate some work
    time.sleep(1)
    _ = [x**2 for x in range(1_000_000)]
