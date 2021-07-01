-- Adds empty space.
local Spacer = {}

local dume = require("dume")
local Vector = require("brinevector")

function Spacer:new(axis, amount)
    local o = {
        params = {
            axis = axis,
            amount = amount,
        },
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Spacer:layout(maxSize)
    self.size = maxSize
    self.size[self.params.axis] = self.params.amount
end

return Spacer
