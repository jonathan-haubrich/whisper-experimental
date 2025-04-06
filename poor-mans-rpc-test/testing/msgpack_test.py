import msgpack

def main():
    with open("round_trip.bin", "rb") as fp:
        encoded = fp.read()

    decoded = msgpack.unpackb(encoded)

    print(f"decoded: {decoded}")

if __name__ == '__main__':
    main()
