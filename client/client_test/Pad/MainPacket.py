# automatically generated by the FlatBuffers compiler, do not modify

# namespace: Pad

import flatbuffers
from flatbuffers.compat import import_numpy
np = import_numpy()

class MainPacket(object):
    __slots__ = ['_tab']

    @classmethod
    def GetRootAs(cls, buf, offset=0):
        n = flatbuffers.encode.Get(flatbuffers.packer.uoffset, buf, offset)
        x = MainPacket()
        x.Init(buf, n + offset)
        return x

    @classmethod
    def GetRootAsMainPacket(cls, buf, offset=0):
        """This method is deprecated. Please switch to GetRootAs."""
        return cls.GetRootAs(buf, offset)
    # MainPacket
    def Init(self, buf, pos):
        self._tab = flatbuffers.table.Table(buf, pos)

    # MainPacket
    def Buttons(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(4))
        if o != 0:
            x = o + self._tab.Pos
            from Pad.ButtonsData import ButtonsData
            obj = ButtonsData()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # MainPacket
    def Lx(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(6))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Uint8Flags, o + self._tab.Pos)
        return 0

    # MainPacket
    def Ly(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(8))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Uint8Flags, o + self._tab.Pos)
        return 0

    # MainPacket
    def Rx(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(10))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Uint8Flags, o + self._tab.Pos)
        return 0

    # MainPacket
    def Ry(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(12))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Uint8Flags, o + self._tab.Pos)
        return 0

    # MainPacket
    def FrontTouch(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(14))
        if o != 0:
            x = self._tab.Indirect(o + self._tab.Pos)
            from Pad.TouchData import TouchData
            obj = TouchData()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # MainPacket
    def BackTouch(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(16))
        if o != 0:
            x = self._tab.Indirect(o + self._tab.Pos)
            from Pad.TouchData import TouchData
            obj = TouchData()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # MainPacket
    def Motion(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(18))
        if o != 0:
            x = o + self._tab.Pos
            from Pad.MotionData import MotionData
            obj = MotionData()
            obj.Init(self._tab.Bytes, x)
            return obj
        return None

    # MainPacket
    def Timestamp(self):
        o = flatbuffers.number_types.UOffsetTFlags.py_type(self._tab.Offset(20))
        if o != 0:
            return self._tab.Get(flatbuffers.number_types.Uint64Flags, o + self._tab.Pos)
        return 0

def MainPacketStart(builder): builder.StartObject(9)
def Start(builder):
    return MainPacketStart(builder)
def MainPacketAddButtons(builder, buttons): builder.PrependStructSlot(0, flatbuffers.number_types.UOffsetTFlags.py_type(buttons), 0)
def AddButtons(builder, buttons):
    return MainPacketAddButtons(builder, buttons)
def MainPacketAddLx(builder, lx): builder.PrependUint8Slot(1, lx, 0)
def AddLx(builder, lx):
    return MainPacketAddLx(builder, lx)
def MainPacketAddLy(builder, ly): builder.PrependUint8Slot(2, ly, 0)
def AddLy(builder, ly):
    return MainPacketAddLy(builder, ly)
def MainPacketAddRx(builder, rx): builder.PrependUint8Slot(3, rx, 0)
def AddRx(builder, rx):
    return MainPacketAddRx(builder, rx)
def MainPacketAddRy(builder, ry): builder.PrependUint8Slot(4, ry, 0)
def AddRy(builder, ry):
    return MainPacketAddRy(builder, ry)
def MainPacketAddFrontTouch(builder, frontTouch): builder.PrependUOffsetTRelativeSlot(5, flatbuffers.number_types.UOffsetTFlags.py_type(frontTouch), 0)
def AddFrontTouch(builder, frontTouch):
    return MainPacketAddFrontTouch(builder, frontTouch)
def MainPacketAddBackTouch(builder, backTouch): builder.PrependUOffsetTRelativeSlot(6, flatbuffers.number_types.UOffsetTFlags.py_type(backTouch), 0)
def AddBackTouch(builder, backTouch):
    return MainPacketAddBackTouch(builder, backTouch)
def MainPacketAddMotion(builder, motion): builder.PrependStructSlot(7, flatbuffers.number_types.UOffsetTFlags.py_type(motion), 0)
def AddMotion(builder, motion):
    return MainPacketAddMotion(builder, motion)
def MainPacketAddTimestamp(builder, timestamp): builder.PrependUint64Slot(8, timestamp, 0)
def AddTimestamp(builder, timestamp):
    return MainPacketAddTimestamp(builder, timestamp)
def MainPacketEnd(builder): return builder.EndObject()
def End(builder):
    return MainPacketEnd(builder)