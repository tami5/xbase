local M = {}
---@param platform Platform
local get_runners = function(platform)
  local devices = {}

  for _, device in ipairs(vim.g.xbase.devices) do
    if device.info.runtime_identifier:match(platform) then
      table.insert(devices, {
        name = device.info.name,
        udid = device.info.udid,
      })
    end
  end

  return devices
end

---Get Targets from project
---To Support Multi Platform targets
---@param project Project
M.get_targets_runners = function(project)
  local targets = {}

  for name, target in pairs(project.targets) do
    if #target.platform > 1 then
      for _, platform in ipairs(target.platform) do
        table.insert(targets, {
          name = string.format("%s_%s", name, platform),
          runners = get_runners(platform),
        })
      end
    else
      table.insert(targets, {
        name = name,
        runners = get_runners(target.platform[1]),
      })
    end
  end

  return targets
end

return M
