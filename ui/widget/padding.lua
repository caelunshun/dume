-- A widget that adds padding around a child.
local Padding = {}

local Vector = require("brinevector")

function Padding:new(child, amount)
    local o = {
        children = { child },
        child = child,
        params = { amount = amount },
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Padding:layout(maxSize, cv)
    local childSize = maxSize - Vector(self.params.amount * 2, self.params.amount * 2)
    self.child:layout(childSize, cv)
    self.child.pos = Vector(self.params.amount, self.params.amount)
    self.size = self.child.size + Vector(self.params.amount * 2, self.params.amount * 2)
    self.offsetFromParent = self.child.offsetFromParent
end

return Padding
