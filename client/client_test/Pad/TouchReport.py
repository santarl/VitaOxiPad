# automatically generated by the FlatBuffers compiler, do not modify

# namespace: Pad

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class TouchReport(object):
    __slots__ = ['_tab']

    @classmethod
    def SizeOf(cls):
        return 6

    # TouchReport
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # TouchReport
    def Pressure(self): return self._tab.Get(flatbuffers.number_types.Uint8Flags, self._tab.Pos + flatbuffers.number_types.UOffsetTFlags.py_type(0))
    # TouchReport
    def Id(self): return self._tab.Get(flatbuffers.number_types.Uint8Flags, self._tab.Pos + flatbuffers.number_types.UOffsetTFlags.py_type(1))
    # TouchReport
    def X(self): return self._tab.Get(flatbuffers.number_types.Int16Flags, self._tab.Pos + flatbuffers.number_types.UOffsetTFlags.py_type(2))
    # TouchReport
    def Y(self): return self._tab.Get(flatbuffers.number_types.Int16Flags, self._tab.Pos + flatbuffers.number_types.UOffsetTFlags.py_type(4))

def CreateTouchReport(builder, pressure, id, x, y):
    builder.Prep(2, 6)
    builder.PrependInt16(y)
    builder.PrependInt16(x)
    builder.PrependUint8(id)
    builder.PrependUint8(pressure)
    return builder.Offset()