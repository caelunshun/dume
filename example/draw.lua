package.path = "ui/?.lua"

local dume = require("dume")
local Vector = require("brinevector")
local Text = require("widget/text")
local Flex = require("widget/flex")

local ui = dume.UI:new(cv)

local root = Text:new("@size{30}{I am @bold{Dume}. @icon{gradient} I am the @italic{%bendu}.}", { bendu = "Bendu" }, {
    alignH = dume.Align.Center,
    alignV = dume.Align.Center
})

ui:createWindow("main", Vector(0, 0), Vector(1920 / 2, 1080 / 2), root)

function draw()
    ui:render()
end
