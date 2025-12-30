import { AppBar, Toolbar, Typography, Button, Box, IconButton } from "@mui/material";
import { Outlet } from "react-router-dom";
import SportsEsportsIcon from "@mui/icons-material/SportsEsports";
import AccountCircleIcon from "@mui/icons-material/AccountCircle";

function Layout() {
  return (
    <Box sx={{ display: "flex", flexDirection: "column", height: "100vh" }}>
      <AppBar position="static" sx={{ bgcolor: "#1b2838" }}>
        <Toolbar>
          <SportsEsportsIcon sx={{ mr: 1 }} />
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            Lutris2
          </Typography>
          <IconButton color="inherit" sx={{ mr: 1 }}>
            <AccountCircleIcon />
          </IconButton>
          <Button color="inherit">Login</Button>
        </Toolbar>
      </AppBar>
      <Box sx={{ flexGrow: 1, overflow: "hidden" }}>
        <Outlet />
      </Box>
    </Box>
  );
}

export default Layout;
