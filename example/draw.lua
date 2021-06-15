package.path = "ui/?.lua"

local dume = require("dume")
local Vector = require("brinevector")
local Text = require("widget/text")
local Flex = require("widget/flex")
local Container = require("widget/container")
local Image = require("widget/image")

local ui = dume.UI:new(cv)

ui.style = {
    backgroundColor = dume.rgb(30, 30, 30),
    borderWidth = 2,
    borderColor = dume.rgb(80, 80, 80),
}

local text1 = Text:new("@size{30}{I am @bold{Dume}. @icon{gradient} I am the @italic{%bendu}.}", { bendu = "Bendu" }, {
    alignH = dume.Align.Center,
})

local text2 = Text:new("@size{14}{@italic{Q.E.D.}}", {}, {
    alignH = dume.Align.Center,
})

local text3 = Text:new("I have spoken.")
local text4 = Text:new("@size{20}{@color{rgb(255,40,40)}{ I am you.}}")

local nested = Flex:row()
nested:addFixedChild(text3)
nested:addFixedChild(text4)

local root = Flex:column()
root:addFlexChild(text1, 1)
root:addFlexChild(text2, 1)
root:addFlexChild(Container:new(nested), 1)
root:addFixedChild(Image:new("smoke", 600))

ui:createWindow("main", Vector(0, 0), Vector(1920 / 2, 1080 / 2), root)

function draw()
    ui:render()
end
