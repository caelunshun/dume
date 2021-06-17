-- A high-level widget for a pressable icon.
local Button = {}

local Container = require("widget/container")
local Clickable = require("widget/clickable")
local StyleModifier = require("widget/style_modifier")

function Button:new(child, pressedCallback)
    local o = {
        children = {
            StyleModifier:new(
                    Clickable:new(
                            Container:new(child),
                            pressedCallback
                    )
            )
        }
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

return Button
