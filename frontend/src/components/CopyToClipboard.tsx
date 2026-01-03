import React, { useState } from 'react';
import { Box, Tooltip, ClickAwayListener } from '@mui/material';

interface CopyToClipboardProps {
  children: React.ReactNode;
  copyText: string;
}

const CopyToClipboard: React.FC<CopyToClipboardProps> = ({ children, copyText }) => {
  const [open, setOpen] = useState(false);

  const handleTooltipClose = () => {
    setOpen(false);
  };

  const handleCopy = (e: React.MouseEvent) => {
    e.stopPropagation(); // prevent modal from closing if someone clicks fast
    navigator.clipboard.writeText(copyText);
    setOpen(true);
    setTimeout(() => {
        setOpen(false);
    }, 1500); // Hide tooltip after 1.5 seconds
  };

  return (
    <ClickAwayListener onClickAway={handleTooltipClose}>
        <Tooltip
          PopperProps={{
            disablePortal: true,
          }}
          onClose={handleTooltipClose}
          open={open}
          disableFocusListener
          disableHoverListener
          disableTouchListener
          title="Copied!"
          arrow
        >
          <Box onClick={handleCopy} sx={{ cursor: 'pointer', display: 'inline-block' }}>
            {children}
          </Box>
        </Tooltip>
    </ClickAwayListener>
  );
};

export default CopyToClipboard;
