# Copyright 2025 The MediaPipe Authors.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
"""MediaPipe image container."""

import ctypes
import enum
from typing import Any

import numpy as np

from mediapipe.tasks.python.core import mediapipe_c_bindings as mediapipe_c_bindings_c_module
from mediapipe.tasks.python.core.optional_dependencies import doc_controls


class ImageFormat(enum.IntEnum):
  """An enum describing supported raw image formats.

  SRGB: sRGB, interleaved: one byte for R, then one byte for G, then one byte
    for B for each pixel.

  SRGBA: sRGBA, interleaved: one byte for R, one byte for G, one byte for B, one
    byte for alpha or unused.

  SBGRA: sBGRA, interleaved: one byte for B, one byte for G, one byte for R, one
    byte for alpha or unused.

  GRAY8: Grayscale, one byte per pixel.

  GRAY16: Grayscale, one uint16 per pixel.

  SRGB48: sRGB, interleaved, each component is a uint16.

  SRGBA64: sRGBA, interleaved, each component is a uint16.

  VEC32F1: One float per pixel.

  VEC32F2: Two floats per pixel.
  """

  UNKNOWN = 0
  SRGB = 1
  SRGBA = 2
  GRAY8 = 3
  GRAY16 = 4
  SRGB48 = 7
  SRGBA64 = 8
  VEC32F1 = 9
  VEC32F2 = 12
  VEC32F4 = 13


def _register_ctypes_signatures(lib: ctypes.CDLL):
  """Registers C function signatures for the given library."""
  lib.MpImageCreateFromUint8Data.argtypes = [
      ctypes.c_int,
      ctypes.c_int,
      ctypes.c_int,
      ctypes.POINTER(ctypes.c_uint8),
      ctypes.c_int,
      ctypes.POINTER(ctypes.c_void_p),
  ]
  lib.MpImageCreateFromUint8Data.restype = ctypes.c_int
  lib.MpImageCreateFromUint16Data.argtypes = [
      ctypes.c_int,
      ctypes.c_int,
      ctypes.c_int,
      ctypes.POINTER(ctypes.c_uint16),
      ctypes.c_int,
      ctypes.POINTER(ctypes.c_void_p),
  ]
  lib.MpImageCreateFromUint16Data.restype = ctypes.c_int
  lib.MpImageCreateFromFloatData.argtypes = [
      ctypes.c_int,
      ctypes.c_int,
      ctypes.c_int,
      ctypes.POINTER(ctypes.c_float),
      ctypes.c_int,
      ctypes.POINTER(ctypes.c_void_p),
  ]
  lib.MpImageCreateFromFloatData.restype = ctypes.c_int
  lib.MpImageCreateFromFile.argtypes = [
      ctypes.c_char_p,
      ctypes.POINTER(ctypes.c_void_p),
  ]
  lib.MpImageCreateFromFile.restype = ctypes.c_int
  lib.MpImageIsContiguous.argtypes = [ctypes.c_void_p]
  lib.MpImageIsContiguous.restype = ctypes.c_bool
  lib.MpImageUsesGpu.argtypes = [ctypes.c_void_p]
  lib.MpImageUsesGpu.restype = ctypes.c_bool
  lib.MpImageIsEmpty.argtypes = [ctypes.c_void_p]
  lib.MpImageIsEmpty.restype = ctypes.c_bool
  lib.MpImageIsAligned.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
  lib.MpImageIsAligned.restype = ctypes.c_bool
  lib.MpImageGetWidth.argtypes = [ctypes.c_void_p]
  lib.MpImageGetWidth.restype = ctypes.c_int
  lib.MpImageGetHeight.argtypes = [ctypes.c_void_p]
  lib.MpImageGetHeight.restype = ctypes.c_int
  lib.MpImageGetChannels.argtypes = [ctypes.c_void_p]
  lib.MpImageGetChannels.restype = ctypes.c_int
  lib.MpImageGetByteDepth.argtypes = [ctypes.c_void_p]
  lib.MpImageGetByteDepth.restype = ctypes.c_int
  lib.MpImageGetWidthStep.argtypes = [ctypes.c_void_p]
  lib.MpImageGetWidthStep.restype = ctypes.c_int
  lib.MpImageGetFormat.argtypes = [ctypes.c_void_p]
  lib.MpImageGetFormat.restype = ctypes.c_int
  lib.MpImageDataUint8.argtypes = [
      ctypes.c_void_p,
      ctypes.POINTER(ctypes.POINTER(ctypes.c_uint8)),
  ]
  lib.MpImageDataUint8.restype = ctypes.c_int
  lib.MpImageDataUint16.argtypes = [
      ctypes.c_void_p,
      ctypes.POINTER(ctypes.POINTER(ctypes.c_uint16)),
  ]
  lib.MpImageDataUint16.restype = ctypes.c_int
  lib.MpImageDataFloat32.argtypes = [
      ctypes.c_void_p,
      ctypes.POINTER(ctypes.POINTER(ctypes.c_float)),
  ]
  lib.MpImageDataFloat32.restype = ctypes.c_int
  lib.MpImageGetValueUint8.argtypes = [
      ctypes.c_void_p,
      ctypes.POINTER(ctypes.c_int),
      ctypes.c_int,
      ctypes.POINTER(ctypes.c_uint8),
  ]
  lib.MpImageGetValueUint8.restype = ctypes.c_int
  lib.MpImageGetValueUint16.argtypes = [
      ctypes.c_void_p,
      ctypes.POINTER(ctypes.c_int),
      ctypes.c_int,
      ctypes.POINTER(ctypes.c_uint16),
  ]
  lib.MpImageGetValueUint16.restype = ctypes.c_int
  lib.MpImageGetValueFloat32.argtypes = [
      ctypes.c_void_p,
      ctypes.POINTER(ctypes.c_int),
      ctypes.c_int,
      ctypes.POINTER(ctypes.c_float),
  ]
  lib.MpImageGetValueFloat32.restype = ctypes.c_int
  lib.MpImageFree.argtypes = [ctypes.c_void_p]
  lib.MpImageFree.restype = None


class Image:
  """A container for storing an image or a video frame.

  Formats supported by Image are listed in the ImageFormat enum.
  Pixels are encoded row-major in an interleaved fashion. Image supports
  uint8, uint16, and float as its data types.

  Image can be created by copying the data from a numpy ndarray that stores
  the pixel data continuously. The data in an Image will become immutable
  after creation.

  The pixel data in an Image can be retrieved as a numpy ndarray by calling
  `Image.numpy_view()`. The returned numpy ndarray is a reference to the
  internal data and itself is unwritable. If the callers want to modify the
  numpy ndarray, it's required to obtain a copy of it.

  Pixel data retrieval examples:

  ```python
  for channel in range(num_channel):
    for col in range(width):
      for row in range(height):
        print(image[row, col, channel])

  output_ndarray = image.numpy_view()
  print(output_ndarray[0, 0, 0])
  copied_ndarray = np.copy(output_ndarray)
  copied_ndarray[0,0,0] = 0
  ```
  """

  _lib: ctypes.CDLL
  _image_ptr: ctypes.c_void_p
  _owned: bool

  def __init__(self, image_format: ImageFormat, data: np.ndarray):
    """Creates an Image object from a numpy ndarray.

    For uint8 data type, valid ImageFormat are GRAY8, SRGB, and SRGBA.
    For uint16 data type, valid ImageFormat are GRAY16, SRGB48, and SRGBA64.
    For float32 data type, valid ImageFormat are VEC32F1, VEC32F2, and VEC32F4.

    Args:
      image_format: The format of the image data.
      data: A numpy ndarray containing the image data.
    """
    lib = mediapipe_c_bindings_c_module.load_shared_library()
    _register_ctypes_signatures(lib)
    self._lib = lib

    height, width, _ = data.shape
    self._image_ptr = ctypes.c_void_p()
    self._owned = True

    if data.dtype == np.uint8:
      status = lib.MpImageCreateFromUint8Data(
          image_format,
          width,
          height,
          data.ctypes.data_as(ctypes.POINTER(ctypes.c_uint8)),
          data.size,
          ctypes.byref(self._image_ptr),
      )
    elif data.dtype == np.uint16:
      status = lib.MpImageCreateFromUint16Data(
          image_format,
          width,
          height,
          data.ctypes.data_as(ctypes.POINTER(ctypes.c_uint16)),
          data.size,
          ctypes.byref(self._image_ptr),
      )
    elif data.dtype == np.float32:
      status = lib.MpImageCreateFromFloatData(
          image_format,
          width,
          height,
          data.ctypes.data_as(ctypes.POINTER(ctypes.c_float)),
          data.size,
          ctypes.byref(self._image_ptr),
      )
    else:
      raise ValueError(f'Unsupported numpy data type: {data.dtype}')

    mediapipe_c_bindings_c_module.handle_status(status)

  @classmethod
  def create_from_file(cls, file_name: str) -> 'Image':
    """Creates an `Image` object from an image file.

    Args:
      file_name: The path to the image file.

    Returns:
      An `Image` object.

    Raises:
      RuntimeError: If the image file cannot be decoded.
    """
    lib = mediapipe_c_bindings_c_module.load_shared_library()
    _register_ctypes_signatures(lib)

    image_ptr = ctypes.c_void_p()
    status = lib.MpImageCreateFromFile(
        file_name.encode('utf-8'), ctypes.byref(image_ptr)
    )
    mediapipe_c_bindings_c_module.handle_status(status)

    # Create an empty Image object and then populate it.
    new_image = cls.__new__(cls)
    new_image._lib = lib
    new_image._image_ptr = image_ptr
    new_image._owned = True
    return new_image

  @classmethod
  @doc_controls.do_not_generate_docs
  def create_from_ctypes(
      cls, image_ptr: ctypes.c_void_p, lib: ctypes.CDLL
  ) -> 'Image':
    """Creates an `Image` object from a ctypes pointer."""
    new_image = cls.__new__(cls)
    new_image._image_ptr = image_ptr
    new_image._lib = lib
    new_image._owned = False
    return new_image

  def numpy_view(self) -> np.ndarray:
    """Returns the image pixel data as an unwritable numpy ndarray.

    Realign the pixel data to be stored contiguously and return a reference to
    the unwritable numpy ndarray. If the callers want to modify the numpy array
    data, it's required to obtain a copy of the ndarray.

    Returns:
      An unwritable numpy ndarray.

    Examples:
      ```
      output_ndarray = image.numpy_view()
      copied_ndarray = np.copy(output_ndarray)
      copied_ndarray[0,0,0] = 0
      ```
    """
    data_ptr = ctypes.POINTER(ctypes.c_uint8)()
    image_format = self.image_format

    if image_format in (ImageFormat.GRAY8, ImageFormat.SRGB, ImageFormat.SRGBA):
      status = self._lib.MpImageDataUint8(
          self._image_ptr, ctypes.byref(data_ptr)
      )
      numpy_ptr = ctypes.cast(data_ptr, ctypes.POINTER(ctypes.c_uint8))
    elif image_format in (
        ImageFormat.GRAY16,
        ImageFormat.SRGB48,
        ImageFormat.SRGBA64,
    ):
      data_ptr = ctypes.POINTER(ctypes.c_uint16)()
      status = self._lib.MpImageDataUint16(
          self._image_ptr, ctypes.byref(data_ptr)
      )
      numpy_ptr = ctypes.cast(data_ptr, ctypes.POINTER(ctypes.c_uint16))
    elif image_format in (
        ImageFormat.VEC32F1,
        ImageFormat.VEC32F2,
        ImageFormat.VEC32F4,
    ):
      data_ptr = ctypes.POINTER(ctypes.c_float)()
      status = self._lib.MpImageDataFloat32(
          self._image_ptr, ctypes.byref(data_ptr)
      )
      numpy_ptr = ctypes.cast(data_ptr, ctypes.POINTER(ctypes.c_float))
    else:
      raise ValueError(f'Unsupported image format: {image_format}')

    mediapipe_c_bindings_c_module.handle_status(status)

    shape = (self.height, self.width, self.channels)
    array = np.ctypeslib.as_array(
        numpy_ptr,
        shape=shape,
    )
    array.flags.writeable = False
    return array

  def __getitem__(self, key: tuple[int, ...]) -> Any:
    """Use the indexer operators to access pixel data.

    Args:
      key: A tuple of integers representing the row, column, and channel indices
        (or row and column for single channel images).

    Returns:
      The pixel data at the specified index.

    Raises:
      IndexError: If the index is invalid or out of bounds.

    Examples:
      ```
      for channel in range(num_channel):
        for col in range(width):
          for row in range(height):
            print(image[row, col, channel])
    ```
    """
    pos_array = (ctypes.c_int * len(key))(*key)
    image_format = self.image_format
    value_ptr = ctypes.c_uint8()

    if image_format in (ImageFormat.GRAY8, ImageFormat.SRGB, ImageFormat.SRGBA):
      status = self._lib.MpImageGetValueUint8(
          self._image_ptr, pos_array, len(key), ctypes.byref(value_ptr)
      )
    elif image_format in (
        ImageFormat.GRAY16,
        ImageFormat.SRGB48,
        ImageFormat.SRGBA64,
    ):
      value_ptr = ctypes.c_uint16()
      status = self._lib.MpImageGetValueUint16(
          self._image_ptr, pos_array, len(key), ctypes.byref(value_ptr)
      )
    elif image_format in (
        ImageFormat.VEC32F1,
        ImageFormat.VEC32F2,
        ImageFormat.VEC32F4,
    ):
      value_ptr = ctypes.c_float()
      status = self._lib.MpImageGetValueFloat32(
          self._image_ptr, pos_array, len(key), ctypes.byref(value_ptr)
      )
    else:
      raise ValueError(f'Unsupported image format: {image_format}')

    mediapipe_c_bindings_c_module.handle_status(status)
    return value_ptr.value

  def uses_gpu(self) -> bool:
    """Return True if data is currently on the GPU.."""
    return self._lib.MpImageUsesGpu(self._image_ptr)

  def is_contiguous(self) -> bool:
    """Return True if the pixel data is stored contiguously (without any alignment padding areas)."""
    return self._lib.MpImageIsContiguous(self._image_ptr)

  def is_empty(self) -> bool:
    """Return True if the pixel data is unallocated."""
    return self._lib.MpImageIsEmpty(self._image_ptr)

  def is_aligned(self, alignment_boundary: int) -> bool:
    """Return True if each row of the data is aligned to alignment boundary, which must be 1 or a power of 2.

    Args:
      alignment_boundary: An integer.

    Returns:
      A boolean.

    Examples:
      ```
      image.is_aligned(16)
      ```
    """
    return self._lib.MpImageIsAligned(self._image_ptr, alignment_boundary)

  @property
  def step(self) -> int:
    """Returns the width step of the image."""
    return self._lib.MpImageGetWidthStep(self._image_ptr)

  @property
  def width(self) -> int:
    """Returns the width of the image."""
    return self._lib.MpImageGetWidth(self._image_ptr)

  @property
  def height(self) -> int:
    """Returns the height of the image."""
    return self._lib.MpImageGetHeight(self._image_ptr)

  @property
  def channels(self) -> int:
    """Returns the number of channels in the image."""
    return self._lib.MpImageGetChannels(self._image_ptr)

  @property
  def image_format(self) -> ImageFormat:
    """Returns the image format."""
    return self._lib.MpImageGetFormat(self._image_ptr)

  def __del__(self):
    """Frees the internal C Image object."""
    if self._owned and self._image_ptr:
      self._lib.MpImageFree(self._image_ptr)
