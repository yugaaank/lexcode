vim.api.nvim_create_user_command("Codelex", function(opts)
  local query = table.concat(opts.fargs, " ")
  local output = vim.fn.systemlist("codelex " .. query)
  local buffer = vim.api.nvim_create_buf(false, true)
  vim.api.nvim_buf_set_lines(buffer, 0, -1, false, output)
  vim.api.nvim_open_win(buffer, true, {
    relative = "editor",
    width = math.floor(vim.o.columns * 0.6),
    height = math.min(#output + 2, 18),
    row = 2,
    col = math.floor(vim.o.columns * 0.2),
    border = "single",
  })
end, { nargs = "*" })
