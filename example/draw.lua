package.path = "ui/?.lua"

local dume = require("dume")
local Vector = require("brinevector")
local Text = require("widget/text")
local Flex = require("widget/flex")

local ui = dume.UI:new(cv)

local text1 = Text:new("@size{30}{I am @bold{Dume}. @icon{gradient} I am the @italic{%bendu}.}", { bendu = "Bendu" }, {
    alignH = dume.Align.Center,
})

local text2 = Text:new("@size{14}{@italic{Q.E.D.}}", {}, {
    alignH = dume.Align.Center,
})

local root = Flex:column()
root:addFlexChild(text1, 1)
root:addFlexChild(text2, 1)

ui:createWindow("main", Vector(0, 0), Vector(1920 / 2, 1080 / 2), root)

function draw()
    ui:render()
end
