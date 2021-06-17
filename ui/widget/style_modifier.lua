-- A widget to change a child's style based on whether
-- it's hovered or pressed.
local StyleModifier = {}

local dume = require("dume")

function StyleModifier:new(child)
    local o = { children = {child}, state = { hovered = false, pressed = false,  }, style = {}, hovered_style = {}, pressed_style = {} }
    o.default_style = o.style
    setmetatable(o, self)
    self.__index = self

    return o
end

function StyleModifier:handleEvent(event, cv)
    if event.type == dume.EventType.MouseClick and self:contains(event.pos) then
        self.state.pressed = event.action == dume.Action.Press
        self:updateStyleForState()
    end

    if event.type == dume.EventType.CursorMove then
        self.state.hovered = self:contains(event.pos)
        self:updateStyleForState()
    end

    self:invokeChildrenEvents(event, cv)
end

function StyleModifier:updateStyleForState()
    self.hovered_style = self.default_style.hovered or {}
    self.pressed_style = self.default_style.pressed or {}

    local index = function(table, key)
        return rawget(table, key) or self.default_style[key]
    end
    self.hovered_style.__index = index
    local pressedIndex = function(table, key)
        return rawget(table, key) or self.hovered_style[key]
    end
    self.pressed_style.__index = pressedIndex
    setmetatable(self.hovered_style, self.hovered_style)
    setmetatable(self.pressed_style, self.pressed_style)

    if self.state.pressed then
        self.style = self.pressed_style
    elseif self.state.hovered then
        self.style = self.hovered_style
    else
        self.style = self.default_style
    end
end

return StyleModifier
