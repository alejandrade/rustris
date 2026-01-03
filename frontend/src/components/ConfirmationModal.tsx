import React from 'react';
import { Modal, Box, Typography, Button } from '@mui/material';

interface ConfirmationModalProps {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  children: React.ReactNode;
  confirmText?: string;
  cancelText?: string;
}

const ConfirmationModal: React.FC<ConfirmationModalProps> = ({
  open,
  onClose,
  onConfirm,
  title,
  children,
  confirmText = 'Confirm',
  cancelText = 'Cancel',
}) => {
  return (
    <Modal
      open={open}
      onClose={onClose}
      aria-labelledby="confirmation-modal-title"
      aria-describedby="confirmation-modal-description"
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
        <Typography id="confirmation-modal-title" sx={{ color: '#ef4444', mb: 2, fontWeight: 'bold', fontSize: '1.25rem' }}>
          {title}
        </Typography>
        <Box id="confirmation-modal-description" sx={{ color: '#d4d4d4', ' & p': { mt: 1 }  }}>
          {children}
        </Box>
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', mt: 3, gap: 2 }}>
          <Button
            onClick={onClose}
            sx={{
              background: '#2a2a2a',
              color: '#ffffff',
              '&:hover': {
                background: '#3a3a3a',
              },
            }}
          >
            {cancelText}
          </Button>
          <Button
            onClick={onConfirm}
            sx={{
              background: '#dc2626',
              color: '#ffffff',
              '&:hover': {
                background: '#b91c1c',
              },
            }}
          >
            {confirmText}
          </Button>
        </Box>
      </Box>
    </Modal>
  );
};

export default ConfirmationModal;
