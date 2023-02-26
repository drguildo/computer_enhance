import random
import sys

def gen_latitude():
    return str(round(random.uniform(-90, 90), 6))

def gen_longitude():
    return str(round(random.uniform(-180, 180), 6))

def gen_data(size):
    print("{\"pairs\":[")
    while size > 0:
        print("{\"x0\":" + gen_longitude() + ", \"y0\":" + gen_latitude() + ", " + "\"x1\":" + gen_longitude() + ", \"y1\":" + gen_latitude() + "}" + ("" if size == 1 else ","))
        size -= 1
    print("]}")

if __name__ == "__main__":
    args = sys.argv[1:]
    if len(args) == 0:
        print("usage: " + sys.argv[0] + " [data set size]")
    else:
        gen_data(int(args[0]))