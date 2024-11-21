from Crypto.Hash import SHA256
from multiprocessing import Pool, cpu_count, set_start_method, Manager
import time

# Constants
UPDATE_INTERVAL = 10_000_000  # Update progress every 10,000,000 iterations


def double_sha256(data):
    """
    Perform double SHA-256 hashing using pycryptodome.
    """
    hash1 = SHA256.new(data).digest()
    hash2 = SHA256.new(hash1).digest()
    return hash2


def worker(payload, start, end, progress_queue):
    """
    Worker process for brute-forcing checksums in a given range.
    """
    for i in range(start, end):
        checksum = i.to_bytes(4, byteorder="big")
        test_payload = payload + checksum

        if i % UPDATE_INTERVAL == 0:
            progress_queue.put((i, time.time()))  # Send progress updates

        if double_sha256(test_payload[:-4])[:4] == checksum:
            progress_queue.put(None)  # Signal completion
            return i, checksum.hex()
    progress_queue.put(None)  # Signal end of worker
    return None, None


def monitor_progress(queue, total_checksums):
    """
    Monitor progress and display real-time updates.
    """
    start_time = time.time()
    processed = 0

    while True:
        try:
            update = queue.get(timeout=1)
            if update is None:  # End signal
                break

            processed, current_time = update
            elapsed_time = current_time - start_time
            progress = processed / total_checksums
            hashes_per_second = processed / elapsed_time if elapsed_time > 0 else 0
            remaining_time = (elapsed_time / progress) - elapsed_time if progress > 0 else 0

            print(
                f"\rCurrent Checksum: {processed:08X} | Progress: {progress * 100:.2f}% | "
                f"Hashes/s: {hashes_per_second:.2f} | Time Remaining: {remaining_time / 60:.2f} min",
                end="",
                flush=True,
            )
        except:
            # Timeout while waiting for updates
            continue


def brute_force(payload):
    """
    Brute-force checksum using multi-processing.
    """
    total_checksums = 0xFFFFFFFF + 1  # Total range of 4-byte checksums
    num_workers = cpu_count()  # Get the number of CPU cores
    chunk_size = total_checksums // num_workers  # Divide range equally among workers

    # Define ranges for each worker
    ranges = [(payload, i, min(i + chunk_size, total_checksums)) for i in range(0, total_checksums, chunk_size)]

    with Manager() as manager:
        progress_queue = manager.Queue()

        # Start multiprocessing pool
        with Pool(processes=num_workers) as pool:
            # Start progress monitoring
            monitor = pool.apply_async(monitor_progress, (progress_queue, total_checksums))
            results = pool.starmap(worker, [(payload, start, end, progress_queue) for (payload, start, end) in ranges])

            monitor.wait()  # Wait for progress monitoring to finish

            for result in results:
                if result[0] is not None:
                    return result
    return None, None


def main():
    """
    Main entry point for the command-line program.
    """
    set_start_method("fork", force=True)  # Use fork for optimal multi-processing on macOS

    # Ask the user for a hex payload
    while True:
        hex_payload = input("Enter the 21-byte payload in HEX format: ").strip()
        try:
            payload = bytes.fromhex(hex_payload)
            if len(payload) != 21:
                print("Invalid payload. Please ensure it is 21 bytes (42 hex characters).")
                continue
            break
        except ValueError:
            print("Invalid hex string. Please enter a valid hexadecimal string.")

    print("Starting brute force...")

    start_time = time.time()

    # Perform brute-force
    checksum_index, checksum = brute_force(payload)

    elapsed_time = time.time() - start_time

    # Display results
    if checksum_index is not None:
        print(f"\nValid checksum found: {checksum} at index {checksum_index}")
    else:
        print("\nNo valid checksum found.")

    print(f"Elapsed time: {elapsed_time:.2f} seconds.")


if __name__ == "__main__":
    import multiprocessing
    main()