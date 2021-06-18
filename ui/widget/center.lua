-- A widget that centers its child in available space.
local Center = {}

function Center:new(child)
    local o = { children = { child }, child = child }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Center:layout(maxSize, cv)
    self.child:layout(maxSize, cv)
    self.child.pos = (maxSize - self.child.size) / 2
    self.size = maxSize
end

return Center
