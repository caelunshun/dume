-- Sets the size of a child.
local Fixed = {}

local Vector = require("brinevector")

function Fixed:new(child, size)
    local o = { child = child, children = {child}, params = { size = size } }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Fixed:layout(maxSize, cv)
    self.child:layout(self.params.size, cv)
    self.size = self.params.size
    self.child.pos = Vector(0, 0)
end

return Fixed
