import React from 'react';
import { Modal, Box, Typography, Button } from '@mui/material';

interface InfoModalProps {
  open: boolean;
  onClose: () => void;
  title: string;
  children: React.ReactNode;
}

const InfoModal: React.FC<InfoModalProps> = ({ open, onClose, title, children }) => {
  return (
    <Modal
      open={open}
      onClose={onClose}
      aria-labelledby="info-modal-title"
      aria-describedby="info-modal-description"
    >
      <Box
        sx={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          maxWidth: 500,
          width: '90%',
          bgcolor: '#141414',
          border: '1px solid #2a2a2a',
          borderRadius: '0.5rem',
          boxShadow: 24,
          p: { xs: 2, sm: 3, md: 4 },
          color: '#ffffff',
          fontFamily: 'system-ui, -apple-system, sans-serif',
        }}
      >
        <Typography id="info-modal-title" sx={{ color: '#8b5cf6', mb: 2, fontWeight: 'bold', fontSize: '1.25rem' }}>
          {title}
        </Typography>
        <Box id="info-modal-description" sx={{ color: '#d4d4d4', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
          {children}
        </Box>
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', mt: 3 }}>
          <Button
            onClick={onClose}
            sx={{
              background: '#8b5cf6',
              color: '#ffffff',
              '&:hover': {
                background: '#7c3aed',
              },
            }}
          >
            Close
          </Button>
        </Box>
      </Box>
    </Modal>
  );
};

export default InfoModal;
