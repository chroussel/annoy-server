import numpy as numpy


def hashcode(a):
    return numpy.int32(a) ^ numpy.int32(a >> 32)


r = hashcode(-311610516216621909)
print(r)
