-- The main widget for composing multiple child widgets.
-- This is a container that lays out its children according
-- to a flexbox model.
local Flex = {}

local dume = require("dume")
local Vector = require("brinevector")

function Flex:row()
    return self:new(dume.Axis.Horizontal)
end

function Flex:column()
    return self:new(dume.Axis.Vertical)
end

function Flex:new(mainAxis)
    local o = {
        mainAxis = mainAxis,
        crossAxis = dume.cross(mainAxis),
        mainAlign = dume.Align.Start,
        crossAlign = dume.Align.Start,
        children = {}
    }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Flex:mainAlign(mainAlign)
    self.params.mainAlign = mainAlign
    return self
end

function Flex:crossAlign(crossAlign)
    self.params.crossAlign = crossAlign
    return self
end

-- Adds a child whose size is fixed.
function Flex:addFixedChild(child)
    self.children[#self.children + 1] = child
    child.isFlex = false
end

-- Adds a child whose size is flexible.
-- The available size among the main axis is divided
-- among all flexible children, weighted with the `flexAmount`
-- parameter.
function Flex:addFlexChild(child, flexAmount)
    self.children[#self.children + 1] = child
    child.isFlex = true
    child.flexAmount = flexAmount
end

function Flex:layout(maxSize, cv)
    -- Compute available space for flex widgets,
    -- as well as the sum of flexAmounts.
    local flexSpace = maxSize[self.mainAxis]
    local flexAmountSum = 0
    for _, widget in ipairs(self.children) do
        if not widget.isFlex then
            widget:layout(maxSize, cv)
            flexSpace = flexSpace - widget.size[self.mainAxis]
        else
            flexAmountSum = flexAmountSum + widget.flexAmount
        end
    end

    -- Lay out widgets.
    local cursor = 0
    for _, widget in ipairs(self.children) do
        if widget.isFlex then
            local mainAxisSpace = widget.flexAmount / flexAmountSum * flexSpace

            local maxWidgetSize = Vector(0, 0)
            maxWidgetSize[self.mainAxis] = mainAxisSpace
            maxWidgetSize[self.crossAxis] = maxSize[self.crossAxis]

            widget:layout(maxWidgetSize, cv)
            widget.pos = Vector(0, 0)
            widget.pos[self.mainAxis] = cursor

            cursor = cursor + mainAxisSpace
        else
            widget.pos = Vector(0, 0)
            widget.pos[self.mainAxis] = cursor
            cursor = cursor + widget.size[self.mainAxis]
        end
    end

    self.size = maxSize
end

return Flex
