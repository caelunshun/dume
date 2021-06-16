-- A widget that detects clicks and invokes a callback.
local Clickable = {}

local Vector = require("brinevector")
local dume = require("dume")

function Clickable:new(child, callback)
    local o = { params = { child = child, callback = callback }, children = { child } }
    setmetatable(o, self)
    self.__index = self
    return o
end

function Clickable:handleEvent(event, cv)
    if event.type == dume.EventType.MouseClick
            and event.action == dume.Action.Press and event.mouse == dume.Mouse.Left
            and self:contains(event.pos) then
        self.params.callback()
    else
        self:invokeChildrenEvents(event, cv)
    end
end

return Clickable
