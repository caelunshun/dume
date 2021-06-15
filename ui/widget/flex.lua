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

function Flex:setMainAlign(mainAlign)
    self.mainAlign = mainAlign
    return self
end

function Flex:setCrossAlign(crossAlign)
    self.crossAlign = crossAlign
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

    -- Apply non-left alignment.
    local totalSize = cursor
    for _, widget in ipairs(self.children) do
        if self.mainAlign == dume.Align.End then
            local start = maxSize[self.mainAxis] - totalSize
            widget.pos[self.mainAxis] = widget.pos[self.mainAxis] + start
        elseif self.mainAlign == dume.Align.Center then
            local mid = maxSize[self.mainAxis] / 2
            local remapped = widget.pos[self.mainAxis] - totalSize / 2
            widget.pos[self.mainAxis] = remapped + mid
        end

        if self.crossAlign == dume.Align.End then
            widget.pos[self.crossAxis] = maxSize.y - widget.size[self.crossAxis]
        elseif self.crossAlign == dume.Align.Center then
            local mid = maxSize[self.crossAxis] / 2
            widget.pos[self.crossAxis] = mid - widget.size[self.crossAxis] / 2
        end
    end

    self.size = maxSize
end

return Flex
